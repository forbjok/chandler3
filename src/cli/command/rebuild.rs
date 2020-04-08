use std::path::Path;

use serde_derive::Serialize;

use chandler::{ChandlerProject, Project};

use super::*;

pub fn rebuild(path: &Path) -> Result<(), CommandError> {
    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct RebuildResult {
        pub input_file_count: i32,
    }

    // Try to load Chandler project
    let mut project = ChandlerProject::load(&path)?;

    project.rebuild()?;

    let result = serde_json::to_string(&RebuildResult { input_file_count: 1 }).unwrap();
    dbg!(result);

    Ok(())
}
