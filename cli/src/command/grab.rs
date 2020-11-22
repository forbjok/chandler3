use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use log::info;

use chandler::{ChandlerProject, Project};

use crate::misc::pathgen;
use crate::ui::*;

use super::*;

pub fn grab(url: &str) -> Result<(), CommandError> {
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

    let mut ui_handler = IndicatifUiHandler::new(Box::new(move || {
        // If cancellation has been requested, break out immediately.
        if cancel.load(Ordering::SeqCst) {
            return true;
        }

        false
    }));

    project
        .update(&mut ui_handler)
        .map_err(|err| CommandError::new(CommandErrorKind::Other, err.to_string()))?;

    project.save()?;
    project.write_thread()?;

    Ok(())
}
