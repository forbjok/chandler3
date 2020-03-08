use url::Url;

use crate::html;

use super::*;

pub fn process_thread<TP>(project: &mut ChandlerProject<TP>, thread_file_path: &Path) -> Result<(), ChandlerError>
where
    TP: MergeableImageboardThread,
{
    let state = &mut project.state;
    let thread_url = Url::parse(&project.config.url)
        .map_err(|err| ChandlerError::Other(format!("Error parsing thread URL: {}", err).into()))?;

    // Parse new thread
    let new_thread = TP::from_file(thread_file_path)?;

    // If there is already a main thread...
    let thread = if let Some(mut original_thread) = project.thread.take() {
        // Merge posts from new thread into the main thread.
        let new_posts = original_thread.merge_posts_from(&new_thread)?;

        // Process links for all new posts.
        for post in new_posts.iter() {
            original_thread.for_post_links(&post, |link| {
                if let Some(link_info) = process_link(link, &thread_url)? {
                    state.links.unprocessed.push(link_info);
                }

                Ok(())
            })?;
        }

        original_thread
    } else {
        // Otherwise...

        // Purge all script tags from the thread HTML.
        new_thread.purge_scripts()?;

        // Process all links in the new thread.
        new_thread.for_links(|link| {
            if let Some(link_info) = process_link(link, &thread_url)? {
                state.links.unprocessed.push(link_info);
            }

            Ok(())
        })?;

        new_thread
    };

    project.thread = Some(thread);

    Ok(())
}

fn process_link(link: &mut html::Link, thread_url: &Url) -> Result<Option<LinkInfo>, ChandlerError> {
    if let Some(href) = link.file_link() {
        if let Some(path) = local_path_from_url(&href, thread_url)? {
            link.replace(&path);

            Ok(Some(LinkInfo { url: href, path }))
        } else {
            return Err(ChandlerError::Other(
                format!("Could not generate local path for url: {}", &href).into(),
            ));
        }
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
