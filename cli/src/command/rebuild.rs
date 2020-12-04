use std::path::Path;

use chandler::project;
use chandler::ui::*;

use crate::error::*;

pub fn rebuild(path: &Path, ui: &mut dyn ChandlerUiHandler) -> Result<(), CliError> {
    // Try to load Chandler project.
    let mut project = project::load(&path)?;

    project.rebuild(ui)?;

    Ok(())
}
