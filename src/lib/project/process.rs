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
            let post_links = original_thread.get_post_links(post)?;

            for link in post_links.iter().filter_map(|l| l.file_link()) {
                state.links.unprocessed.push(link);
            }
        }

        original_thread
    } else {
        let links = new_thread.get_links()?;

        for link in links.iter().filter_map(|l| l.file_link()) {
            state.links.unprocessed.push(link);
        }

        new_thread
    };

    project.thread = Some(thread);

    Ok(())
}
