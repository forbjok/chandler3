use crate::threadparser::MergeableImageboardThread;
use crate::threadparser::fourchan::FourchanThread;

use super::*;

pub struct FourchanThreadUpdater {
    thread: FourchanThread,
}

impl FourchanThreadUpdater {
    pub fn from_file(file_path: &Path) -> Result<Self, ChandlerError> {
        Ok(Self {
            thread: FourchanThread::from_file(file_path)?,
        })
    }
}

impl ThreadUpdater for FourchanThreadUpdater {
    fn perform_initial_cleanup(&mut self) -> Result<UpdateResult, ChandlerError> {
        // Purge all script tags from the thread HTML.
        self.thread.purge_scripts()?;

        let mut new_links: Vec<html::Link> = Vec::new();

        // Process all links in the new thread.
        self.thread.for_links(|link| {
            new_links.push(link);

            Ok(())
        })?;

        let is_archived = self.thread.is_archived()?;

        Ok(UpdateResult {
            is_archived,
            new_links,
        })
    }

    fn update_from(&mut self, path: &Path) -> Result<UpdateResult, ChandlerError> {
        let thread = &mut self.thread;

        // Parse new thread.
        let new_thread = FourchanThread::from_file(path)?;

        // Merge posts from new thread into the main thread.
        let new_posts = thread.merge_posts_from(&new_thread)?;

        let mut new_links: Vec<html::Link> = Vec::new();

        // Process links for all new posts.
        for post in new_posts.iter() {
            thread.for_post_links(&post, |link| {
                new_links.push(link);

                Ok(())
            })?;
        }

        let is_archived = new_thread.is_archived()?;

        Ok(UpdateResult {
            is_archived,
            new_links,
        })
    }

    fn write_file(&self, file_path: &Path) -> Result<(), ChandlerError> {
        self.thread.write_file(file_path)
    }
}
