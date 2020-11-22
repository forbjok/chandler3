use std::io;
use std::path::Path;

use log::info;

use chandler::util;
use chandler::{ChandlerProject, Project};

use crate::result::*;
use crate::ui::*;

use super::*;

pub fn grab(url: &str, destination: &Path) -> Result<(), CommandError> {
    let project_path = util::normalize_path(destination);

    info!("Project path: {}", project_path.display());

    let mut project = ChandlerProject::load_or_create(&project_path, url)
        .map_err(|err| CommandError::new(CommandErrorKind::Other, err.to_string()))?;

    let mut ui_handler = StderrUiHandler::new();

    let update_result = project
        .update(&mut ui_handler)
        .map_err(|err| CommandError::new(CommandErrorKind::Other, err.to_string()))?;

    project.save()?;
    project.write_thread()?;

    let stdout = io::stdout();

    serde_json::to_writer_pretty(
        stdout,
        &PcliUpdateResult {
            url: url.to_owned(),
            destination_path: project_path,
            was_updated: update_result.was_updated,
            is_dead: update_result.is_dead,
            new_post_count: update_result.new_post_count,
            new_file_count: update_result.new_file_count,
        },
    )
    .map_err(|err| CommandError::new(CommandErrorKind::Other, err.to_string()))?;

    Ok(())
}
