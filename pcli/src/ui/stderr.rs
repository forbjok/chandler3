use chandler::ui::*;

pub struct StderrUiHandler;

impl StderrUiHandler {
    pub fn new() -> Self {
        Self
    }
}

impl ChandlerUiHandler for StderrUiHandler {
    fn event(&mut self, e: &UiEvent) {
        match e {
            UiEvent::DownloadStart { file_count } => {
                eprintln!("Downloading {} files...", file_count);
            }
            UiEvent::DownloadProgress { .. } => {}
            UiEvent::DownloadComplete {
                files_downloaded,
                files_failed,
            } => {
                eprintln!(
                    "Download finished. {} files downloaded, {} files failed.",
                    files_downloaded, files_failed
                );
            }
            UiEvent::DownloadFileStart { url, .. } => {
                eprintln!("Downloading {} ...", url);
            }
            UiEvent::DownloadFileInfo { .. } => {}
            UiEvent::DownloadFileProgress { .. } => {}
            UiEvent::DownloadFileComplete(r) => {
                match r {
                    DownloadFileCompleteResult::Success => eprintln!("Download file complete."),
                    DownloadFileCompleteResult::NotModified => eprintln!("File not modified. Download unnecessary."),
                    DownloadFileCompleteResult::Error => eprintln!("Download file failed!"),
                };
            }

            UiEvent::UpdateStart { thread_url, .. } => {
                eprintln!("Updating thread from {}...", thread_url);
            }
            UiEvent::UpdateComplete {
                was_updated,
                new_post_count,
                new_file_count,
            } => {
                if *was_updated {
                    eprintln!(
                        "Update finished. {} new posts, {} new files.",
                        new_post_count, new_file_count
                    );
                } else {
                    eprintln!("Thread not changed.");
                }
            }

            UiEvent::RebuildStart { .. } => {}
            UiEvent::RebuildProgress { .. } => {}
            UiEvent::RebuildComplete => {}
        }
    }

    fn is_cancelled(&self) -> bool {
        false
    }
}
