use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use log::info;

use chandler::project;

use crate::misc::pathgen;
use crate::ui::*;
use crate::ProjectOptions;

use super::*;

pub fn grab(url: &str, project_options: &ProjectOptions) -> Result<(), CommandError> {
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

    let mut project = project::load_or_create(project_path, url, project_options.into())?;

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

    Ok(())
}
