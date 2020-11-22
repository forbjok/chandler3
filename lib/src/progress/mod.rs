use std::path::PathBuf;

#[derive(Debug)]
pub enum DownloadCompleteResult {
    Success,
    NotModified,
    Error,
}

#[derive(Debug)]
pub enum ProgressEvent {
    DownloadStart { file_count: u32 },
    DownloadProgress { files_processed: u32 },
    DownloadComplete { files_downloaded: u32, files_failed: u32 },

    DownloadFileStart { url: String, destination: PathBuf },
    DownloadFileInfo { size: Option<u64> },
    DownloadFileProgress { bytes_downloaded: u64 },
    DownloadFileComplete(DownloadCompleteResult),

    UpdateStart { thread_url: String, destination: PathBuf },
    UpdateComplete { new_post_count: i32 },
}

pub trait ChandlerProgressCallbackHandler {
    fn progress(&mut self, e: &ProgressEvent);
}
