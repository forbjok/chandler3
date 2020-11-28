use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};
use log::info;

use chandler::project;

use crate::ui::*;
use crate::ProjectOptions;

use super::*;

const ONE_SECOND: Duration = Duration::from_secs(1);

use lazy_static::lazy_static;

lazy_static! {
    static ref WAITING_BAR_STYLE: ProgressStyle = ProgressStyle::default_bar().template(" {prefix} {pos} {wide_msg}");
}

pub fn watch(url: &str, interval: i64, project_options: &ProjectOptions) -> Result<(), CommandError> {
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

    let mut project = project::builder()
        .url(url)
        .use_chandler_config()?
        .use_sites_config()?
        .format(project_options.format.into())
        .load_or_create()?;

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

    let waiting_bar = ProgressBar::new(wait_seconds)
        .with_style((*WAITING_BAR_STYLE).clone())
        .with_prefix("Waiting")
        .with_message(message)
        .with_position(wait_seconds);

    // Draw initial bar.
    waiting_bar.tick();

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
