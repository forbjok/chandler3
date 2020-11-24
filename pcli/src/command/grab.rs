use std::io;
use std::path::Path;

use log::info;

use chandler::project::{self, ProjectUpdateResult};
use chandler::util;

use crate::result::*;
use crate::ui::*;
use crate::ProjectFormat;

use super::*;

pub fn grab(url: &str, destination: &Path, format: ProjectFormat) -> Result<(), CommandError> {
    let project_path = util::normalize_path(destination);

    info!("Project path: {}", project_path.display());

    let project_format = match format {
        ProjectFormat::V2 => project::ProjectFormat::V2,
        ProjectFormat::V3 => project::ProjectFormat::V3,
    };

    let result = (|| -> Result<ProjectUpdateResult, PcliError> {
        let mut project = project::load_or_create_format(&project_path, url, project_format)?;

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
