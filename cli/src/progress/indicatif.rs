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
    static ref DOWNLOAD_SPINNER_STYLE: ProgressStyle =
        ProgressStyle::default_spinner().template(" {prefix:>8} {bytes}/? {wide_msg}");
}

pub struct IndicatifProgressHandler {
    overall_download_bar: Option<ProgressBar>,
    file_download_bar: Option<ProgressBar>,
    rebuild_bar: Option<ProgressBar>,
}

impl IndicatifProgressHandler {
    pub fn new() -> Self {
        Self {
            overall_download_bar: None,
            file_download_bar: None,
            rebuild_bar: None,
        }
    }
}

impl<'a> ChandlerProgressCallbackHandler for IndicatifProgressHandler {
    fn progress(&mut self, e: &ProgressEvent) {
        match e {
            ProgressEvent::DownloadStart { file_count } => {
                let bar = ProgressBar::new(*file_count as u64).with_style((*OVERALL_DOWNLOAD_BAR_STYLE).clone());

                bar.set_prefix("Overall");
                bar.set_message("files downloaded...");

                self.overall_download_bar = Some(bar);
            }
            ProgressEvent::DownloadProgress { files_processed } => {
                if let Some(bar) = &self.overall_download_bar {
                    bar.set_position(*files_processed as u64);
                }
            }
            ProgressEvent::DownloadComplete {
                files_downloaded,
                files_failed,
            } => {
                if let Some(bar) = self.overall_download_bar.take() {
                    bar.println(format!(
                        "Download finished. {} files downloaded, {} files failed.",
                        files_downloaded, files_failed
                    ));
                    bar.finish_and_clear();
                }
            }
            ProgressEvent::DownloadFileStart { url, .. } => {
                let bar = ProgressBar::new(0).with_style((*DOWNLOAD_SPINNER_STYLE).clone());
                bar.set_prefix("Download");
                bar.set_message(&url);

                self.file_download_bar = Some(bar);
            }
            ProgressEvent::DownloadFileInfo { size } => {
                if let Some(bar) = &self.file_download_bar {
                    if let Some(size) = *size {
                        bar.set_style((*DOWNLOAD_BAR_STYLE).clone());
                        bar.set_length(size);
                    }
                }
            }
            ProgressEvent::DownloadFileProgress { bytes_downloaded } => {
                if let Some(bar) = &self.file_download_bar {
                    bar.set_position(*bytes_downloaded);
                }
            }
            ProgressEvent::DownloadFileComplete(_) => {
                if let Some(bar) = &self.file_download_bar.take() {
                    bar.finish_and_clear();
                }
            }

            ProgressEvent::UpdateStart { .. } => {}
            ProgressEvent::UpdateComplete { .. } => {}

            ProgressEvent::RebuildStart { path, file_count } => {
                println!("Rebuilding project at {}...", path.display());

                let bar = ProgressBar::new(*file_count as u64).with_style((*OVERALL_DOWNLOAD_BAR_STYLE).clone());

                bar.set_prefix("Rebuild");

                self.rebuild_bar = Some(bar);
            }
            ProgressEvent::RebuildProgress { files_processed } => {
                if let Some(bar) = &self.rebuild_bar {
                    bar.set_position(*files_processed as u64);
                }
            }
            ProgressEvent::RebuildComplete => {
                if let Some(bar) = &self.rebuild_bar.take() {
                    bar.println("Rebuild finished.");
                    bar.finish_and_clear();
                }
            }
        }
    }
}
