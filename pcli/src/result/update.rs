use std::path::PathBuf;

use serde_derive::Serialize;

#[derive(Debug, Serialize)]
pub struct PcliUpdateResult {
    pub url: String,
    pub destination_path: PathBuf,
    pub was_updated: bool,
    pub is_dead: bool,
    pub new_post_count: u32,
    pub new_file_count: u32,
}
