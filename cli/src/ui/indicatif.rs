use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use chandler::ui::*;

pub struct IndicatifUiHandler {
    multi_progress: MultiProgress,

    progress_chars: String,
    cancel_check: Box<dyn Fn() -> bool>,

    overall_download_pb: Option<ProgressBar>,
    file_download_pb: Option<ProgressBar>,
    rebuild_pb: Option<ProgressBar>,
}

impl IndicatifUiHandler {
    pub fn new(progress_chars: String, cancel_check: Box<dyn Fn() -> bool>) -> Self {
        Self {
            multi_progress: MultiProgress::new(),

            progress_chars,
            cancel_check,

            overall_download_pb: None,
            file_download_pb: None,
            rebuild_pb: None,
        }
    }
}

impl ChandlerUiHandler for IndicatifUiHandler {
    fn event(&mut self, e: &UiEvent) {
        match e {
            UiEvent::DownloadStart { file_count } => {
                let pb = ProgressBar::new(*file_count as u64)
                    .with_style(
                        ProgressStyle::default_bar()
                            .template(" {prefix:>8} [{bar:40.cyan/blue}] {pos}/{len} {wide_msg}")
                            .unwrap()
                            .progress_chars(&self.progress_chars),
                    )
                    .with_prefix("Overall")
                    .with_message("files downloaded...");

                let pb = self.multi_progress.add(pb);

                self.overall_download_pb = Some(pb);
            }
            UiEvent::DownloadProgress { files_processed } => {
                if let Some(pb) = &self.overall_download_pb {
                    pb.set_position(*files_processed as u64);
                }
            }
            UiEvent::DownloadComplete {
                files_downloaded,
                files_failed,
            } => {
                if let Some(pb) = self.overall_download_pb.take() {
                    pb.println(format!(
                        "Download finished. {} files downloaded, {} files failed.",
                        files_downloaded, files_failed
                    ));
                    pb.finish_and_clear();
                }
            }
            UiEvent::DownloadFileStart { url, .. } => {
                let pb = ProgressBar::new(0)
                    .with_style(
                        ProgressStyle::default_spinner()
                            .template(" {prefix:>8} {bytes}/? {wide_msg}")
                            .unwrap(),
                    )
                    .with_prefix("Download")
                    .with_message(url.to_owned());

                let pb = self.multi_progress.add(pb);

                self.file_download_pb = Some(pb);
            }
            UiEvent::DownloadFileInfo { size } => {
                if let Some(pb) = &self.file_download_pb {
                    if let Some(size) = *size {
                        pb.set_style(
                            ProgressStyle::default_bar()
                                .template(" {prefix:>8} [{bar:40.cyan/blue}] {bytes}/{total_bytes} {wide_msg}")
                                .unwrap()
                                .progress_chars(&self.progress_chars),
                        );
                        pb.set_length(size);
                    }
                }
            }
            UiEvent::DownloadFileProgress { bytes_downloaded } => {
                if let Some(pb) = &self.file_download_pb {
                    pb.set_position(*bytes_downloaded);
                }
            }
            UiEvent::DownloadFileComplete(_) => {
                if let Some(pb) = &self.file_download_pb.take() {
                    pb.finish_and_clear();
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

                let pb = ProgressBar::new(*file_count as u64)
                    .with_style(
                        ProgressStyle::default_bar()
                            .template(" {prefix:>8} [{bar:40.cyan/blue}] {pos}/{len} {wide_msg}")
                            .unwrap()
                            .progress_chars(&self.progress_chars),
                    )
                    .with_prefix("Rebuild");

                let pb = self.multi_progress.add(pb);

                self.rebuild_pb = Some(pb);
            }
            UiEvent::RebuildProgress { files_processed } => {
                if let Some(pb) = &self.rebuild_pb {
                    pb.set_position(*files_processed as u64);
                }
            }
            UiEvent::RebuildComplete => {
                if let Some(pb) = &self.rebuild_pb.take() {
                    pb.println("Rebuild finished.");
                    pb.finish_and_clear();
                }
            }
        }
    }

    fn is_cancelled(&self) -> bool {
        (self.cancel_check)()
    }

    fn clear(&mut self) {
        self.multi_progress.clear().unwrap();
    }
}
