use crate::threadparser::basic::BasicThread;
use crate::threadparser::HtmlDocument;

use super::*;

pub struct BasicThreadUpdater {
    thread: BasicThread,
}

impl BasicThreadUpdater {
    pub fn from_file(file_path: &Path) -> Result<Self, ChandlerError> {
        Ok(Self {
            thread: BasicThread::from_file(file_path)?,
        })
    }
}

impl ThreadUpdater for BasicThreadUpdater {
    fn perform_initial_cleanup(&mut self) -> Result<UpdateResult, ChandlerError> {
        // Purge all script tags from the thread HTML.
        self.thread.purge_scripts()?;

        let mut new_links: Vec<html::Link> = Vec::new();

        // Process all links in the new thread.
        self.thread.for_links(|link| {
            new_links.push(link);

            Ok(())
        })?;

        Ok(UpdateResult {
            is_archived: false,
            new_post_count: 0,
            new_links,
        })
    }

    fn update_from(&mut self, path: &Path) -> Result<UpdateResult, ChandlerError> {
        // Parse new thread.
        self.thread = BasicThread::from_file(path)?;

        let mut new_links: Vec<html::Link> = Vec::new();

        // Process all links in the new thread.
        self.thread.for_links(|link| {
            new_links.push(link);

            Ok(())
        })?;

        Ok(UpdateResult {
            is_archived: false,
            new_post_count: 0,
            new_links,
        })
    }

    fn write_file(&self, file_path: &Path) -> Result<(), ChandlerError> {
        self.thread.write_file(file_path)
    }
}
