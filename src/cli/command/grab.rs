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

    project.update()
        .map_err(|err| CommandError::new(CommandErrorKind::Other, err.to_string()))?;

    project.save()?;
    project.write_thread()?;

    Ok(())
}
