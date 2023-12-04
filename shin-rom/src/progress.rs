use std::time::Instant;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use crate::index::{DirectoryIterCtx, EntryContent};

#[derive(Default)]
struct RomCounter {
    files: u64,
    directories: u64,
    bytes: u64,
}

impl RomCounter {
    fn add(&mut self, entry: &EntryContent) {
        match entry {
            EntryContent::File(content) => {
                self.files += 1;
                self.bytes += content.len() as u64;
            }
            EntryContent::Directory(_) => self.directories += 1,
        }
    }
}

pub struct ExtractProgress {
    start_time: Instant,
    running_counter: RomCounter,
    #[allow(unused)] // we need to keep the multi progress alive
    multi_progress: MultiProgress,
    file_progress: ProgressBar,
    size_progress: ProgressBar,
}

impl ExtractProgress {
    pub fn new(ctx: &DirectoryIterCtx) -> Self {
        let mut total_counter = RomCounter::default();
        super::iter_rom(&ctx, |_, entry| total_counter.add(entry));

        let multi_progress = MultiProgress::new();
        let file_progress = multi_progress.add(
            ProgressBar::new(total_counter.files).with_style(
                ProgressStyle::default_bar()
                    .template("files: [{bar:40.cyan/blue}] {human_pos}/{human_len} ({per_sec})")
                    .unwrap()
                    .progress_chars("#>-"),
            ),
        );
        let size_progress = multi_progress.add(ProgressBar::new(total_counter.bytes)
            .with_style(ProgressStyle::default_bar()
                .template("bytes: [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({binary_bytes_per_sec})")
                .unwrap()
                .progress_chars("#>-")));

        multi_progress.add(file_progress.clone());
        multi_progress.add(size_progress.clone());

        ExtractProgress {
            start_time: Instant::now(),
            running_counter: RomCounter::default(),
            multi_progress,
            file_progress,
            size_progress,
        }
    }

    pub fn add(&mut self, entry: &EntryContent) {
        self.running_counter.add(entry);

        self.file_progress.set_position(self.running_counter.files);
        self.size_progress.set_position(self.running_counter.bytes);
    }

    pub fn finish(&self) {
        self.file_progress.finish();
        self.size_progress.finish();

        let elapsed = self.start_time.elapsed();

        let files_per_sec = self.running_counter.files as f64 / elapsed.as_secs_f64();
        let bytes_per_sec = self.running_counter.bytes as f64 / elapsed.as_secs_f64();

        eprintln!(
            "Extracted {} files and {} bytes in {:.2}s ({} files/s, {}/s)",
            self.running_counter.files,
            bytesize::ByteSize(self.running_counter.bytes),
            elapsed.as_secs_f64(),
            files_per_sec as u64,
            bytesize::ByteSize(bytes_per_sec as u64),
        );
    }
}
