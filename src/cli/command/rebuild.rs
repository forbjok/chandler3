use std::path::Path;

use serde_derive::Serialize;

use chandler::{ChandlerProject, Project};

use super::*;

pub fn rebuild(path: &Path) -> Result<String, CommandError> {
    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct RebuildResult {
        pub input_file_count: i32,
    }

    // Try to load Chandler project
    let project = ChandlerProject::load(&path)?;

    let result = project.rebuild();

    match result {
        Ok(_path) => { },
        Err(_msg) => { },
    };

    Ok(serde_json::to_string(&RebuildResult {
        input_file_count: 1,
    }).unwrap())
}
