use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use log::info;

use chandler::{ChandlerProject, Project};

use super::*;
use crate::misc::pathgen;

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

    dbg!(&project_path);

    let mut project = ChandlerProject::load_or_create(project_path, url)
        .map_err(|err| CommandError::new(CommandErrorKind::Other, err.to_string()))?;

    let one_second = Duration::from_secs(1);
    let interval = chrono::Duration::seconds(interval);

    // Cancellation boolean.
    let cancel = Arc::new(AtomicBool::new(false));

    let break_cancel = cancel.clone();

    // Set break (Ctrl-C) handler.
    ctrlc::set_handler(move || {
        info!("Cancellation requested by user.");
        break_cancel.store(true, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let mut next_update_at: DateTime<Utc>;

    'watch: loop {
        info!("Updating thread...");

        let update_result = project
            .update(cancel.clone())
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

            std::thread::sleep(one_second);
        }
    }

    Ok(())
}
