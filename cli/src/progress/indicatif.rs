use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use lazy_static::lazy_static;

use chandler::progress::{ChandlerProgressCallbackHandler, ProgressEvent};

lazy_static! {
    static ref DOWNLOAD_BAR_STYLE: ProgressStyle = ProgressStyle::default_bar()
        .template(" {prefix:>8} [{elapsed_precise}] [{bar:40}] {bytes}/{total_bytes} {wide_msg}")
        .progress_chars("=> ");

    static ref DOWNLOAD_SPINNER_STYLE: ProgressStyle = ProgressStyle::default_spinner()
        .template(" {prefix:>8} [{elapsed_precise}] {spinner} {bytes} {wide_msg}");
}

pub struct IndicatifProgressHandler {
    download_bar: ProgressBar,
}

impl IndicatifProgressHandler {
    pub fn new(multi_progress: &MultiProgress) -> Self {
        let download_bar = multi_progress.add(ProgressBar::new_spinner());

        Self {
            download_bar,
        }
    }
}

impl ChandlerProgressCallbackHandler for IndicatifProgressHandler {
    fn progress(&mut self, e: &ProgressEvent) {
        match e {
            ProgressEvent::DownloadStart(dsi) => {
                let bar = &self.download_bar;

                bar.set_style((*DOWNLOAD_SPINNER_STYLE).clone());
                bar.set_prefix("Downloading");
                bar.set_message(&dsi.url);
            },
            ProgressEvent::DownloadFileInfo(dfi) => {
                let bar = &self.download_bar;

                if let Some(size) = dfi.size {
                    bar.set_style((*DOWNLOAD_BAR_STYLE).clone());
                    bar.set_length(size);
                }
            },
            ProgressEvent::DownloadProgress(dpi) => {
                self.download_bar.set_position(dpi.bytes_downloaded);
            },
            ProgressEvent::DownloadComplete(_) => self.download_bar.set_style((*DOWNLOAD_SPINNER_STYLE).clone()),
        }
    }
}
