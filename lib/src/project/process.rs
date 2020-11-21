use std::collections::HashSet;

use url::Url;

use crate::html;
use crate::threadupdater::UpdateResult;

use super::*;

pub fn process_thread(project: &mut ChandlerProject, thread_file_path: &Path) -> Result<UpdateResult, ChandlerError> {
    let state = &mut project.state;
    let extensions: HashSet<String> = project.config.download_extensions.iter().cloned().collect();

    let thread_url = Url::parse(&project.config.url)
        .map_err(|err| ChandlerError::Other(format!("Error parsing thread URL: {}", err).into()))?;

    // If there is already a main thread...
    let mut update_result = if let Some(original_thread) = project.thread.as_mut() {
        original_thread.update_from(thread_file_path)?
    } else {
        // Otherwise...

        // Parse new thread
        let mut new_thread = project.config.parser.create_thread_updater_from(thread_file_path)?;
        let update_result = new_thread.perform_initial_cleanup()?;

        project.thread = Some(new_thread);

        update_result
    };

    // Process new links.
    for link in update_result.new_links.iter_mut() {
        if let Some(link_info) = process_link(link, &thread_url, &extensions)? {
            state.links.unprocessed.push(link_info);
        }
    }

    Ok(update_result)
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
                    let absolute_url = thread_url.join(&href).unwrap();

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

fn local_path_from_url(url_str: &str, thread_url: &Url) -> Result<Option<String>, ChandlerError> {
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
