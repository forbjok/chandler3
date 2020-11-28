use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use log::info;

use chandler::project;

use crate::ui::*;
use crate::ProjectOptions;

use super::*;

pub fn grab(url: &str, project_options: &ProjectOptions) -> Result<(), CommandError> {
    let mut project = project::builder()
        .url(url)
        .use_chandler_config()?
        .use_sites_config()?
        .format(project_options.format.into())
        .load_or_create()?;

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
