use std::path::PathBuf;

#[derive(Debug)]
pub enum DownloadFileCompleteResult {
    Success,
    NotModified,
    Error,
}

#[derive(Debug)]
pub enum UiEvent {
    // Overall download operation.
    DownloadStart { file_count: u32 },
    DownloadProgress { files_processed: u32 },
    DownloadComplete { files_downloaded: u32, files_failed: u32 },

    // File download operation.
    DownloadFileStart { url: String, destination: PathBuf },
    DownloadFileInfo { size: Option<u64> },
    DownloadFileProgress { bytes_downloaded: u64 },
    DownloadFileComplete(DownloadFileCompleteResult),

    // Update thread operation.
    UpdateStart { thread_url: String, destination: PathBuf },
    UpdateComplete { new_post_count: i32 },

    // Rebuild thread operation.
    RebuildStart { path: PathBuf, file_count: u32 },
    RebuildProgress { files_processed: u32 },
    RebuildComplete,
}

pub trait ChandlerUiHandler {
    fn event(&mut self, e: &UiEvent);
}
