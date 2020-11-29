use crate::threadparser::MergeableImageboardThread;

use super::*;

pub struct MergingThreadUpdater<TP: MergeableImageboardThread> {
    thread: TP,
}

impl<TP: MergeableImageboardThread> MergingThreadUpdater<TP> {
    pub fn from_file(file_path: &Path) -> Result<Self, ChandlerError> {
        Ok(Self {
            thread: TP::from_file(file_path)?,
        })
    }
}

impl<TP: MergeableImageboardThread> ThreadUpdater for MergingThreadUpdater<TP> {
    fn perform_initial_cleanup(&mut self) -> Result<UpdateResult, ChandlerError> {
        // Purge all script tags from the thread HTML.
        self.thread.purge_scripts()?;

        let new_post_count = (1 + self.thread.get_all_replies().map_or(0, |iter| iter.count())) as u32;

        let mut new_links: Vec<html::Link> = Vec::new();

        // Process all links in the new thread.
        self.thread.for_links(|link| {
            new_links.push(link);

            Ok(())
        })?;

        let is_archived = self.thread.is_archived()?;

        Ok(UpdateResult {
            is_archived,
            new_post_count,
            new_links,
        })
    }

    fn update_from(&mut self, path: &Path) -> Result<UpdateResult, ChandlerError> {
        let thread = &mut self.thread;

        // Parse new thread.
        let new_thread = TP::from_file(path)?;

        let is_archived = new_thread.is_archived()?;

        // Merge posts from new thread into the main thread.
        let new_replies = thread.merge_replies_from(new_thread)?;
        let new_post_count = new_replies.len() as u32;

        let mut new_links: Vec<html::Link> = Vec::new();

        // Process links for all new replies.
        for reply in new_replies.iter() {
            thread.for_reply_links(&reply, |link| {
                new_links.push(link);

                Ok(())
            })?;
        }

        Ok(UpdateResult {
            is_archived,
            new_post_count,
            new_links,
        })
    }

    fn write_file(&self, file_path: &Path) -> Result<(), ChandlerError> {
        self.thread.write_file(file_path)
    }
}
