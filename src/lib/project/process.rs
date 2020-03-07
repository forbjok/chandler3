use super::*;

pub fn process_thread<TP>(project: &mut ChandlerProject<TP>, thread_file_path: &Path) -> Result<(), ChandlerError>
where
    TP: MergeableImageboardThread,
{
    let state = &mut project.state;

    // Parse new thread
    let new_thread = TP::from_file(thread_file_path)?;

    let thread = if let Some(mut original_thread) = project.thread.take() {
        let new_posts = original_thread.merge_posts_from(&new_thread)?;

        for post in new_posts.iter() {
            original_thread.for_post_links(&post, |link| {
                if let Some(href) = link.file_link() {
                    state.links.unprocessed.push(href);
                    link.replace("--FILE--");
                } else {
                    link.replace("");
                }
            })?;
        }

        original_thread
    } else {
        new_thread.for_links(|link| {
            if let Some(href) = link.file_link() {
                state.links.unprocessed.push(href);
                link.replace("--FILE--");
            } else {
                link.replace("");
            }
        })?;

        new_thread
    };

    project.thread = Some(thread);

    Ok(())
}
