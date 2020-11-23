use serde_derive::Serialize;

use chandler::project::ProjectUpdateResult;

#[derive(Debug, Serialize)]
pub struct PcliUpdateResult {
    pub was_updated: bool,
    pub is_dead: bool,
    pub new_post_count: u32,
    pub new_file_count: u32,
}

impl From<ProjectUpdateResult> for PcliUpdateResult {
    fn from(ur: ProjectUpdateResult) -> Self {
        PcliUpdateResult {
            was_updated: ur.was_updated,
            is_dead: ur.is_dead,
            new_post_count: ur.new_post_count,
            new_file_count: ur.new_file_count,
        }
    }
}
