use chandler::project;
use chandler::ui::*;

use crate::{GeneralOptions, ProjectOptions};

use crate::error::*;

pub fn grab(
    url: &str,
    general_options: &GeneralOptions,
    project_options: &ProjectOptions,
    ui: &mut dyn ChandlerUiHandler,
) -> Result<(), CliError> {
    let mut project = project::builder()
        .url(url)
        .config_path(general_options.config_path.as_deref())
        .use_chandler_config(true)?
        .use_sites_config(true)?
        .format(Some(project_options.format.into()))
        .load_or_create()?;

    eprintln!("Project path: {}", project.get_path().display());

    project
        .update(ui)
        .map_err(|err| CliError::new(CliErrorKind::Other, err.to_string()))?;

    project.save()?;

    Ok(())
}
