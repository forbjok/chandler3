use std::path::PathBuf;

#[derive(Debug)]
pub struct DownloadStartInfo {
    pub url: String,
    pub destination: PathBuf,
}

#[derive(Debug)]
pub struct DownloadFileInfo {
    pub size: Option<u64>,
}

#[derive(Debug)]
pub struct DownloadProgressInfo {
    pub bytes_downloaded: u64,
}

#[derive(Debug)]
pub enum DownloadCompleteResult {
    Success,
    NotModified,
    Error,
}

#[derive(Debug)]
pub struct DownloadCompleteInfo {
    pub result: DownloadCompleteResult,
}

#[derive(Debug)]
pub enum ProgressEvent {
    DownloadStart(DownloadStartInfo),
    DownloadFileInfo(DownloadFileInfo),
    DownloadProgress(DownloadProgressInfo),
    DownloadComplete(DownloadCompleteInfo),
}

pub trait ChandlerProgressCallbackHandler {
    fn progress(&mut self, e: &ProgressEvent);
}
