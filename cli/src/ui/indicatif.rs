use indicatif::{ProgressBar, ProgressStyle};

use chandler::ui::*;

pub struct IndicatifUiHandler {
    progress_chars: String,
    cancel_check: Box<dyn Fn() -> bool>,

    overall_download_bar: Option<ProgressBar>,
    file_download_bar: Option<ProgressBar>,
    rebuild_bar: Option<ProgressBar>,
}

impl IndicatifUiHandler {
    pub fn new(progress_chars: String, cancel_check: Box<dyn Fn() -> bool>) -> Self {
        Self {
            progress_chars,
            cancel_check,

            overall_download_bar: None,
            file_download_bar: None,
            rebuild_bar: None,
        }
    }
}

impl ChandlerUiHandler for IndicatifUiHandler {
    fn event(&mut self, e: &UiEvent) {
        match e {
            UiEvent::DownloadStart { file_count } => {
                let bar = ProgressBar::new(*file_count as u64)
                    .with_style(
                        ProgressStyle::default_bar()
                            .template(" {prefix:>8} [{bar:40.cyan/blue}] {pos}/{len} {wide_msg}")
                            .progress_chars(&self.progress_chars),
                    )
                    .with_prefix("Overall")
                    .with_message("files downloaded...");

                // Draw initial bar.
                bar.tick();

                self.overall_download_bar = Some(bar);
            }
            UiEvent::DownloadProgress { files_processed } => {
                if let Some(bar) = &self.overall_download_bar {
                    bar.set_position(*files_processed as u64);
                }
            }
            UiEvent::DownloadComplete {
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
            UiEvent::DownloadFileStart { url, .. } => {
                let bar = ProgressBar::new(0)
                    .with_style(ProgressStyle::default_spinner().template(" {prefix:>8} {bytes}/? {wide_msg}"))
                    .with_prefix("Download")
                    .with_message(&url);

                // Draw initial bar.
                bar.tick();

                self.file_download_bar = Some(bar);
            }
            UiEvent::DownloadFileInfo { size } => {
                if let Some(bar) = &self.file_download_bar {
                    if let Some(size) = *size {
                        bar.set_style(
                            ProgressStyle::default_bar()
                                .template(" {prefix:>8} [{bar:40.cyan/blue}] {bytes}/{total_bytes} {wide_msg}")
                                .progress_chars(&self.progress_chars),
                        );
                        bar.set_length(size);
                    }
                }
            }
            UiEvent::DownloadFileProgress { bytes_downloaded } => {
                if let Some(bar) = &self.file_download_bar {
                    bar.set_position(*bytes_downloaded);
                }
            }
            UiEvent::DownloadFileComplete(_) => {
                if let Some(bar) = &self.file_download_bar.take() {
                    bar.finish_and_clear();
                }
            }

            UiEvent::UpdateStart { thread_url, .. } => {
                eprintln!("Updating thread from {}...", thread_url);
            }
            UiEvent::UpdateError { description } => {
                eprintln!("Update thread failed: {}", description);
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

            UiEvent::RebuildStart { path, file_count } => {
                println!("Rebuilding project at {}...", path.display());

                let bar = ProgressBar::new(*file_count as u64)
                    .with_style(
                        ProgressStyle::default_bar()
                            .template(" {prefix:>8} [{bar:40.cyan/blue}] {pos}/{len} {wide_msg}")
                            .progress_chars(&self.progress_chars),
                    )
                    .with_prefix("Rebuild");

                self.rebuild_bar = Some(bar);
            }
            UiEvent::RebuildProgress { files_processed } => {
                if let Some(bar) = &self.rebuild_bar {
                    bar.set_position(*files_processed as u64);
                }
            }
            UiEvent::RebuildComplete => {
                if let Some(bar) = &self.rebuild_bar.take() {
                    bar.println("Rebuild finished.");
                    bar.finish_and_clear();
                }
            }
        }
    }

    fn is_cancelled(&self) -> bool {
        (self.cancel_check)()
    }
}
