use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};
use log::info;

use chandler::project;

use crate::misc::pathgen;
use crate::ui::*;

use super::*;

const ONE_SECOND: Duration = Duration::from_secs(1);

use lazy_static::lazy_static;

lazy_static! {
    static ref WAITING_BAR_STYLE: ProgressStyle = ProgressStyle::default_bar().template(" {prefix} {pos} {wide_msg}");
}

pub fn watch(url: &str, interval: i64) -> Result<(), CommandError> {
    let config = crate::config::CliConfig::from_default_location()
        .map_err(|err| CommandError::new(CommandErrorKind::Config, Cow::Owned(err)))?
        .resolve()
        .map_err(|err| CommandError::new(CommandErrorKind::Config, Cow::Owned(err)))?;

    let sites_config = crate::config::load_sites_config()?;

    let project_path = pathgen::generate_destination_path(&config, &sites_config, url).map_err(|err| {
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

    let mut project = project::load_or_create(project_path, &url)?;

    let ui_cancel = cancel.clone();
    let mut ui_handler = IndicatifUiHandler::new(Box::new(move || {
        // If cancellation has been requested, break out immediately.
        if ui_cancel.load(Ordering::SeqCst) {
            return true;
        }

        false
    }));

    'watch: loop {
        let update_result = {
            let result = project.update(&mut ui_handler);

            if let Err(ChandlerError::Download(_)) = &result {
                // Wait for retry.
                if !waiting_bar(interval_seconds, "seconds until retry...", &cancel) {
                    // If user requested cancellation, break out of the loop.
                    break 'watch;
                }

                continue 'watch;
            }

            result.map_err(|err| CommandError::new(CommandErrorKind::Other, err.to_string()))?
        };

        // Save changes to disk.
        project.save()?;

        // If the thread is dead, break out of loop.
        if update_result.is_dead {
            eprintln!("Thread is dead.");
            break 'watch;
        }

        // Wait for next update.
        if !waiting_bar(interval_seconds, "seconds until update...", &cancel) {
            // If user requested cancellation, break out of the loop.
            break 'watch;
        }
    }

    Ok(())
}

/// Wait with progress indicator.
fn waiting_bar(wait_seconds: u64, message: &str, cancel: &Arc<AtomicBool>) -> bool {
    let mut seconds_passed: u64 = 0;

    let waiting_bar = ProgressBar::new(wait_seconds);
    waiting_bar.set_style((*WAITING_BAR_STYLE).clone());
    waiting_bar.set_prefix("Waiting");
    waiting_bar.set_message(message);
    waiting_bar.set_position(wait_seconds);

    // Wait until the specified time has passed.
    while seconds_passed < wait_seconds {
        // If cancellation has been requested, break out immediately.
        if cancel.load(Ordering::SeqCst) {
            return false;
        }

        std::thread::sleep(ONE_SECOND);

        // Update waiting progress.
        seconds_passed += 1;
        waiting_bar.set_position(wait_seconds - seconds_passed);
    }

    waiting_bar.finish_and_clear();

    true
}
