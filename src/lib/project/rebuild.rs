use std::borrow::Cow;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use kuchiki;

use crate::error::*;
use crate::threadparser::*;
use crate::util;

fn parse_html_file(filename: &Path) -> Result<kuchiki::NodeRef, ChandlerError> {
    use std::fs::File;

    use html5ever::parse_document;
    use html5ever::driver::ParseOpts;
    use html5ever::tendril::TendrilSink;
    use html5ever::tree_builder::TreeBuilderOpts;

    let opts = ParseOpts {
        tree_builder: TreeBuilderOpts {
            drop_doctype: false,
            ..Default::default()
        },
        ..Default::default()
    };

    let mut f = util::open_file(filename).map_err(|err| ChandlerError::OpenFile(err))?;

    let dom = kuchiki::parse_html()
        .from_utf8()
        .read_from(&mut f)
        .map_err(|err| ChandlerError::ReadFile(err))?;

    Ok(dom)
}

pub fn rebuild_thread(files: &[PathBuf], destination_file: &Path) -> Result<(), ChandlerError> {
    // Get file iterator
    let mut files_iter = files.iter();

    // Get the first file
    let first_file = files_iter.next()
        .ok_or_else(|| ChandlerError::Other(Cow::Owned("First file not found!".to_owned())))?;

    let first_dom = parse_html_file(first_file)?;
    let mut first_thread = fourchan::FourchanThread::from_document(first_dom);

    let mut first_thread_posts = first_thread.get_all_posts()
        .map_err(|err| ChandlerError::Other(Cow::Owned("Couldn't get first thread posts!".to_owned())))?;

    println!("Thread no. {}", first_thread_posts.next().expect("First post not found in first thread!").id);

    for file in files_iter {
        println!("FILE: {:?}", file);

        let dom = parse_html_file(file)?;

        let thread = fourchan::FourchanThread::from_document(dom);

        first_thread.merge_posts_from(thread)
            .map_err(|err| ChandlerError::Other(Cow::Owned(err.to_string())))?;
    }

    let mut outfile = util::create_file(destination_file).map_err(|err| ChandlerError::CreateFile(err))?;

    let first_dom = first_thread.into_document();

    html5ever::serialize(&mut outfile, &first_dom, Default::default())
        .ok()
        .expect("Serialization failed");

    Ok(())
}
