use std::path::Path;

use url::Url;

use crate::error::*;
use crate::html;
use crate::project::ProjectState;
use crate::threadupdater::{CreateThreadUpdater, UpdateResult};

#[derive(Debug)]
pub struct LinkInfo {
    pub url: String,
    pub path: String,
}

#[derive(Debug)]
pub struct ProcessResult {
    pub update_result: UpdateResult,
}

pub fn process_thread(state: &mut ProjectState, new_thread_file_path: &Path) -> Result<ProcessResult, ChandlerError> {
    // If there is already a main thread...
    let (thread, mut update_result) = if let Some(mut original_thread) = state.thread.take() {
        let update_result = original_thread.update_from(new_thread_file_path)?;

        (original_thread, update_result)
    } else {
        // Otherwise...

        // Parse new thread
        let mut new_thread = state.parser.create_thread_updater_from(new_thread_file_path)?;
        let update_result = new_thread.perform_initial_cleanup()?;

        (new_thread, update_result)
    };

    // Put thread in project state.
    state.thread = Some(thread);

    // Process new links.
    for link in update_result.new_links.iter_mut() {
        if let Some(link_info) = process_link(state, link)? {
            state.new_links.push(link_info);
        }
    }

    Ok(ProcessResult { update_result })
}

fn process_link(state: &mut ProjectState, link: &mut html::Link) -> Result<Option<LinkInfo>, ChandlerError> {
    if let Some(href) = link.file_link() {
        let thread_url = Url::parse(&state.thread_url)
            .map_err(|err| ChandlerError::Other(format!("Error parsing thread URL: {}", err).into()))?;

        // Make URL absolute.
        let absolute_url = thread_url.join(&href).map_err(|err| {
            ChandlerError::Other(format!("Error making URL '{}' absolute: {}", &href, err.to_string()).into())
        })?;

        // Make file URL with query and fragment removed.
        let file_url = {
            let mut url = absolute_url.clone();
            url.set_query(None);
            url.set_fragment(None);

            url.to_string()
        };

        if let Some(extension) = file_url.rsplit('.').next() {
            if state.download_extensions.contains(extension) {
                if let Some(path) = state.link_path_generator.generate_path(&file_url)? {
                    link.replace(&path);

                    return Ok(Some(LinkInfo {
                        url: absolute_url.into_string(),
                        path,
                    }));
                } else {
                    return Err(ChandlerError::Other(
                        format!("Could not generate local path for url: {}", &href).into(),
                    ));
                }
            }
        }

        Ok(None)
    } else {
        Ok(None)
    }
}
