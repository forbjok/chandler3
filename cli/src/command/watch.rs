use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use chrono::{DateTime, Utc};
use indicatif::MultiProgress;
use log::info;

use chandler::{ChandlerProject, Project};
use chandler::progress::ProgressEvent;

use crate::misc::pathgen;
use crate::progress::IndicatifProgressHandler;

use super::*;

const ONE_SECOND: Duration = Duration::from_secs(1);

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


    let multi_progress = MultiProgress::new();
    let mut progress_handler = IndicatifProgressHandler::new(&multi_progress);

    let url = url.to_owned();

    // Spawn watcher thread
    let result = thread::spawn(move || {
        let mut project = ChandlerProject::load_or_create(project_path, &url)
        .map_err(|err| CommandError::new(CommandErrorKind::Other, err.to_string()))?;

        let interval = chrono::Duration::seconds(interval);

        let mut next_update_at: DateTime<Utc>;

        'watch: loop {
            info!("Updating thread...");

            let update_result = project
                .update(cancel.clone(), &mut progress_handler)
                .map_err(|err| CommandError::new(CommandErrorKind::Other, err.to_string()))?;

            // Save changes to disk.
            project.save()?;
            project.write_thread()?;

            // If the thread is dead, break out of loop.
            if update_result.is_dead {
                info!("Thread is dead.");
                break;
            }

            // Calculate next update time.
            next_update_at = Utc::now() + interval;

            info!("Next update at {}.", next_update_at);

            // Wait for until it's time for the next update.
            while Utc::now() < next_update_at {
                // If cancellation has been requested, break out immediately.
                if cancel.load(Ordering::SeqCst) {
                    break 'watch;
                }

                std::thread::sleep(ONE_SECOND);
            }
        }

        Ok(())
    });

    multi_progress.join().unwrap();

    result.join().unwrap()
}
