use chandler::project;
use chandler::ui::*;

use crate::ProjectOptions;

use crate::error::*;

pub fn grab(url: &str, project_options: &ProjectOptions, ui: &mut dyn ChandlerUiHandler) -> Result<(), CliError> {
    let mut project = project::builder()
        .url(url)
        .use_chandler_config()?
        .use_sites_config()?
        .format(project_options.format.into())
        .load_or_create()?;

    eprintln!("Project path: {}", project.get_path().display());

    project
        .update(ui)
        .map_err(|err| CliError::new(CliErrorKind::Other, err.to_string()))?;

    project.save()?;

    Ok(())
}
