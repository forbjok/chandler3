use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use chrono::{DateTime, Utc};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use log::info;

use chandler::{ChandlerProject, Project};

use crate::misc::pathgen;
use crate::progress::IndicatifProgressHandler;

use super::*;

const ONE_SECOND: Duration = Duration::from_secs(1);

use lazy_static::lazy_static;

lazy_static! {
    static ref WAITING_BAR_STYLE: ProgressStyle = ProgressStyle::default_bar()
        .template(" {prefix} {pos} {wide_msg}");
}

pub fn watch(url: &str, interval: i64) -> Result<(), CommandError> {
    let config = crate::config::CliConfig::from_default_location()
        .map_err(|err| CommandError::new(CommandErrorKind::Config, Cow::Owned(err)))?
        .resolve()
        .map_err(|err| CommandError::new(CommandErrorKind::Config, Cow::Owned(err)))?;

    let project_path = pathgen::generate_destination_path(&config, url).map_err(|err| {
        CommandError::new(
            CommandErrorKind::Other,
            format!("Could not generate path for url '{}': {}", url, err),
        )
    })?;

    info!("Project path: {}", project_path.display());

    // Cancellation boolean.
    let cancel = Arc::new(AtomicBool::new(false));

    let break_cancel = cancel.clone();

    // Set break (Ctrl-C) handler.
    ctrlc::set_handler(move || {
        info!("Cancellation requested by user.");
        break_cancel.store(true, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let interval_seconds = interval as u64;

    let mut project = ChandlerProject::load_or_create(project_path, &url)
        .map_err(|err| CommandError::new(CommandErrorKind::Other, err.to_string()))?;

    let interval_duration = chrono::Duration::seconds(interval);

    let mut progress_handler = IndicatifProgressHandler::new();

    let mut next_update_at: DateTime<Utc>;

    'watch: loop {
        println!("Updating thread... ");

        let update_result = project
            .update(cancel.clone(), &mut progress_handler)
            .map_err(|err| CommandError::new(CommandErrorKind::Other, err.to_string()))?;

        // Save changes to disk.
        project.save()?;
        project.write_thread()?;

        println!("Update finished.");

        // If the thread is dead, break out of loop.
        if update_result.is_dead {
            println!("Thread is dead.");
            break;
        }

        // Calculate next update time.
        next_update_at = Utc::now() + interval_duration;

        info!("Next update at {}.", next_update_at);

        let mut seconds_passed: u64 = 0;

        let waiting_bar = ProgressBar::new(interval_seconds);
        waiting_bar.set_style((*WAITING_BAR_STYLE).clone());
        waiting_bar.set_prefix("Waiting");
        waiting_bar.set_message("seconds until update...");
        waiting_bar.set_position(interval_seconds);

        // Wait for until it's time for the next update.
        while Utc::now() < next_update_at {
            // If cancellation has been requested, break out immediately.
            if cancel.load(Ordering::SeqCst) {
                break 'watch;
            }

            std::thread::sleep(ONE_SECOND);

            // Update waiting progress.
            seconds_passed += 1;
            waiting_bar.set_position(interval_seconds - seconds_passed);
        }
    }

    Ok(())
}
