use std::path::Path;

use log::debug;
use serde_derive::Serialize;

use chandler::project;

use crate::ui::*;

use super::*;

pub fn rebuild(path: &Path) -> Result<(), CommandError> {
    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct RebuildResult {
        pub input_file_count: i32,
    }

    // Try to load Chandler project
    let mut project = project::load(&path)?;

    let mut ui_handler = IndicatifUiHandler::new(Box::new(|| false));

    project.rebuild(&mut ui_handler)?;

    let result = serde_json::to_string(&RebuildResult { input_file_count: 1 }).unwrap();
    debug!("Result: {:#?}", result);

    Ok(())
}
