use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use lazy_static::lazy_static;

use chandler::progress::{ChandlerProgressCallbackHandler, ProgressEvent};

lazy_static! {
    static ref OVERALL_DOWNLOAD_BAR_STYLE: ProgressStyle = ProgressStyle::default_bar()
        .template(" {prefix:>8} [{bar:40.cyan/blue}] {pos}/{len} {wide_msg}")
        .progress_chars("##-");

    static ref DOWNLOAD_BAR_STYLE: ProgressStyle = ProgressStyle::default_bar()
        .template(" {prefix:>8} [{bar:40.cyan/blue}] {bytes}/{total_bytes} {wide_msg}")
        .progress_chars("##-");

    static ref DOWNLOAD_SPINNER_STYLE: ProgressStyle = ProgressStyle::default_spinner()
        .template(" {prefix:>8} {spinner} {bytes} {wide_msg}");
}

pub struct IndicatifProgressHandler {
    overall_download_bar: ProgressBar,
    file_download_bar: ProgressBar,
}

impl IndicatifProgressHandler {
    pub fn new(multi_progress: &MultiProgress) -> Self {
        let overall_download_bar = multi_progress.add(ProgressBar::new(0))
            .with_style((*OVERALL_DOWNLOAD_BAR_STYLE).clone());
        overall_download_bar.set_prefix("Overall");
        overall_download_bar.set_message("files downloaded...");

        let file_download_bar = multi_progress.add(ProgressBar::new(0));
        file_download_bar.set_prefix("Download");

        Self {
            overall_download_bar,
            file_download_bar,
        }
    }
}

impl ChandlerProgressCallbackHandler for IndicatifProgressHandler {
    fn progress(&mut self, e: &ProgressEvent) {
        match e {
            ProgressEvent::DownloadStart { file_count } => {
                self.overall_download_bar.set_length(*file_count as u64);
                self.overall_download_bar.set_position(0);
            },
            ProgressEvent::DownloadProgress { files_processed } => {
                self.overall_download_bar.set_position(*files_processed as u64);
            },
            ProgressEvent::DownloadComplete { .. } => { },
            ProgressEvent::DownloadFileStart { url, .. } => {
                let bar = &self.file_download_bar;

                bar.set_style((*DOWNLOAD_SPINNER_STYLE).clone());
                bar.set_message(&url);
            },
            ProgressEvent::DownloadFileInfo { size } => {
                let bar = &self.file_download_bar;

                if let Some(size) = *size {
                    bar.set_style((*DOWNLOAD_BAR_STYLE).clone());
                    bar.set_length(size);
                }
            },
            ProgressEvent::DownloadFileProgress { bytes_downloaded } => {
                self.file_download_bar.set_position(*bytes_downloaded);
            },
            ProgressEvent::DownloadFileComplete(_) => self.file_download_bar.set_style((*DOWNLOAD_SPINNER_STYLE).clone()),

            ProgressEvent::UpdateStart { .. } => { },
            ProgressEvent::UpdateComplete { .. } => { },
        }
    }
}
