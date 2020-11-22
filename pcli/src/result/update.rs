use std::path::PathBuf;

use serde_derive::Serialize;

use chandler::UpdateResult;

#[derive(Debug, Serialize)]
pub struct PcliUpdateResult {
    pub was_updated: bool,
    pub is_dead: bool,
    pub new_post_count: u32,
    pub new_file_count: u32,
}

impl From<UpdateResult> for PcliUpdateResult {
    fn from(ur: UpdateResult) -> Self {
        PcliUpdateResult {
            was_updated: ur.was_updated,
            is_dead: ur.is_dead,
            new_post_count: ur.new_post_count,
            new_file_count: ur.new_file_count,
        }
    }
}
