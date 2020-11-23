use std::io;
use std::path::Path;

use log::info;

use chandler::project::{self, ProjectUpdateResult};
use chandler::util;

use crate::result::*;
use crate::ui::*;

use super::*;

pub fn grab(url: &str, destination: &Path) -> Result<(), CommandError> {
    let project_path = util::normalize_path(destination);

    info!("Project path: {}", project_path.display());

    let result = (|| -> Result<ProjectUpdateResult, PcliError> {
        let mut project = project::load_or_create(&project_path, url)?;

        let mut ui_handler = StderrUiHandler::new();

        let update_result = project.update(&mut ui_handler)?;

        project.save()?;

        Ok(update_result)
    })();

    let pcli_result: PcliResult<PcliUpdateResult> = result.into();

    let stdout = io::stdout();

    serde_json::to_writer_pretty(stdout, &pcli_result)
        .map_err(|err| CommandError::new(CommandErrorKind::Other, err.to_string()))?;

    Ok(())
}
