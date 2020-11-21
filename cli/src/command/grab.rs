use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use log::info;

use chandler::{ChandlerProject, Project};

use super::*;

pub fn grab(url: &str) -> Result<(), CommandError> {
    let config = crate::config::CliConfig::from_default_location()
        .map_err(|err| CommandError::new(CommandErrorKind::Config, Cow::Owned(err)))?
        .resolve()
        .map_err(|err| CommandError::new(CommandErrorKind::Config, Cow::Owned(err)))?;

    let project_path = config.save_to_path.join("new_thread_placeholder");

    dbg!(&project_path);

    let mut project = ChandlerProject::load_or_create(project_path, url)
        .map_err(|err| CommandError::new(CommandErrorKind::Other, err.to_string()))?;

    // Cancellation boolean.
    let cancel = Arc::new(AtomicBool::new(false));

    let break_cancel = cancel.clone();

    // Set break (Ctrl-C) handler.
    ctrlc::set_handler(move || {
        info!("Cancellation requested by user.");
        break_cancel.store(true, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    project
        .update(cancel)
        .map_err(|err| CommandError::new(CommandErrorKind::Other, err.to_string()))?;

    project.save()?;
    project.write_thread()?;

    Ok(())
}
