use std::collections::HashSet;
use std::path::Path;

use log::debug;
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
    let thread_url = Url::parse(&state.thread_url)
        .map_err(|err| ChandlerError::Other(format!("Error parsing thread URL: {}", err).into()))?;

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
        if let Some(link_info) = process_link(link, &thread_url, &state.download_extensions)? {
            state.new_links.push(link_info);
        }
    }

    Ok(ProcessResult { update_result })
}

fn process_link(
    link: &mut html::Link,
    thread_url: &Url,
    extensions: &HashSet<String>,
) -> Result<Option<LinkInfo>, ChandlerError> {
    if let Some(href) = link.file_link() {
        if let Some(extension) = href.rsplit('.').next() {
            if extensions.contains(extension) {
                if let Some(path) = local_path_from_url(&href, thread_url)? {
                    link.replace(&path);

                    // Make URL absolute.
                    let absolute_url = thread_url.join(&href).map_err(|err| {
                        ChandlerError::Other(
                            format!("Error making URL '{}' absolute: {}", &href, err.to_string()).into(),
                        )
                    })?;

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

pub fn local_path_from_url(url_str: &str, thread_url: &Url) -> Result<Option<String>, ChandlerError> {
    let url = thread_url
        .join(url_str)
        .map_err(|err| ChandlerError::Other(err.to_string().into()))?;

    if let Some(host) = url.host_str() {
        Ok(Some(format!("{}{}", host, url.path())))
    } else {
        debug!("No host found in url: {}", url_str);
        Ok(None)
    }
}
