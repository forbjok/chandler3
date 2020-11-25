use std::path::Path;

use chandler::project;

use crate::ui::*;

use super::*;

pub fn rebuild(path: &Path) -> Result<(), CommandError> {
    // Try to load Chandler project
    let mut project = project::load(&path)?;

    let mut ui_handler = IndicatifUiHandler::new(Box::new(|| false));

    project.rebuild(&mut ui_handler)?;

    Ok(())
}
