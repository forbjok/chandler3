use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

use chandler::error::*;
use chandler::project;
use chandler::ui::*;

use crate::{GeneralOptions, ProjectOptions};

use crate::error::*;

const ONE_SECOND: Duration = Duration::from_secs(1);

pub fn watch(
    url: &str,
    interval: i64,
    general_options: &GeneralOptions,
    project_options: &ProjectOptions,
    ui: &mut dyn ChandlerUiHandler,
) -> Result<(), CliError> {
    let mut project = project::builder()
        .url(url)
        .config_path(general_options.config_path.as_deref())
        .use_chandler_config(true)?
        .use_sites_config(true)?
        .format(Some(project_options.format.into()))
        .load_or_create()?;

    eprintln!("Project path: {}", project.get_path().display());

    let interval_seconds = interval as u64;

    'watch: loop {
        let update_result = {
            let result = project.update(ui);

            if let Err(ChandlerError::Download(_)) = &result {
                // Wait for retry.
                if !waiting_bar(interval_seconds, "seconds until retry...", ui) {
                    // If user requested cancellation, break out of the loop.
                    break 'watch;
                }

                continue 'watch;
            }

            result.map_err(|err| CliError::new(CliErrorKind::Other, err.to_string()))?
        };

        // Save changes to disk.
        project.save()?;

        // If the thread is dead, break out of loop.
        if update_result.is_dead {
            eprintln!("Thread is dead.");
            break 'watch;
        }

        // Wait for next update.
        if !waiting_bar(interval_seconds, "seconds until update...", ui) {
            // If user requested cancellation, break out of the loop.
            break 'watch;
        }
    }

    Ok(())
}

/// Wait with progress indicator.
fn waiting_bar(wait_seconds: u64, message: &str, ui: &mut dyn ChandlerUiHandler) -> bool {
    let mut seconds_passed: u64 = 0;

    let waiting_bar = ProgressBar::new(wait_seconds)
        .with_style(
            ProgressStyle::default_bar()
                .template(" {prefix} {pos} {wide_msg}")
                .unwrap(),
        )
        .with_prefix("Waiting")
        .with_message(message.to_owned())
        .with_position(wait_seconds);

    // Draw initial bar.
    waiting_bar.tick();

    // Wait until the specified time has passed.
    while seconds_passed < wait_seconds {
        // If cancellation has been requested, break out immediately.
        if ui.is_cancelled() {
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
