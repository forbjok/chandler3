use scraper::Html;

fn for_links_mut(
    html: &mut Html,
    mut action: impl FnMut(html::Link) -> Result<(), ChandlerError>,
) -> Result<(), ChandlerError> {
    let links = html::find_links(self.html.clone());

    for link in links.into_iter() {
        action(link)?;
    }

    Ok(())
}
