use std::path::Path;

use chandler::project;

use crate::ui::*;

use crate::error::*;

pub fn rebuild(path: &Path) -> Result<(), CliError> {
    // Try to load Chandler project
    let mut project = project::load(&path)?;

    let mut ui_handler = IndicatifUiHandler::new(Box::new(|| false));

    project.rebuild(&mut ui_handler)?;

    Ok(())
}
