//! Progress reporting utilities.

use std::time::Instant;

use indicatif::ProgressStyle;
use tracing::{info, info_span, span::EnteredSpan};
use tracing_indicatif::span_ext::IndicatifSpanExt;

#[derive(Default, Copy, Clone)]
pub struct RomCounter {
    pub directories: u64,
    pub files: u64,
    pub bytes: u64,
}

impl RomCounter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_file(&mut self, size: u64) {
        self.files += 1;
        self.bytes += size;
    }
    pub fn add_directory(&mut self) {
        self.directories += 1;
    }
}

pub enum ProgressAction {
    Extract,
    Create,
}

pub struct RomTimingSummary {
    action: ProgressAction,
    start_time: Instant,
}

impl RomTimingSummary {
    pub fn new(action: ProgressAction) -> Self {
        RomTimingSummary {
            action,
            start_time: Instant::now(),
        }
    }

    pub fn finish(self, total_counter: RomCounter) {
        let elapsed = self.start_time.elapsed();

        let files_per_sec = total_counter.files as f64 / elapsed.as_secs_f64();
        let bytes_per_sec = total_counter.bytes as f64 / elapsed.as_secs_f64();

        let action_text = match self.action {
            ProgressAction::Extract => "Extracted",
            ProgressAction::Create => "Packed",
        };

        info!(
            "{} {} files and {} bytes in {:.2}s ({} files/s, {}/s)",
            action_text,
            total_counter.files,
            bytesize::ByteSize(total_counter.bytes),
            elapsed.as_secs_f64(),
            files_per_sec as u64,
            bytesize::ByteSize(bytes_per_sec as u64),
        );
    }
}

/// Manages two spans that display progress of ROM processing: file-wise and byte-wise.
///
/// This is useful because, depending on file size, the bottleneck is either the number of files or the total size of files, and typical game roms contain both large and small files.
pub struct RomProgress {
    running_counter: RomCounter,
    file_progress_span: EnteredSpan,
    size_progress_span: EnteredSpan,
}

impl RomProgress {
    pub fn new(total_counter: RomCounter) -> Self {
        let file_progress_span = info_span!("files");
        file_progress_span.pb_set_style(
            &ProgressStyle::default_bar()
                .template("{span_child_prefix}files: [{bar:40.cyan/blue}] {human_pos}/{human_len} ({per_sec})")
                .unwrap()
                .progress_chars("#>-"),
        );
        file_progress_span.pb_set_length(total_counter.files);

        let size_progress_span = info_span!("bytes");
        size_progress_span.pb_set_style(
            &ProgressStyle::default_bar()
                .template(
                    "{span_child_prefix}bytes: [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({binary_bytes_per_sec})",
                )
                .unwrap()
                .progress_chars("#>-"),
        );
        size_progress_span.pb_set_length(total_counter.bytes);

        RomProgress {
            running_counter: RomCounter::default(),
            file_progress_span: file_progress_span.entered(),
            size_progress_span: size_progress_span.entered(),
        }
    }

    pub fn add_file(&mut self, size: u64) {
        self.running_counter.add_file(size);

        self.file_progress_span
            .pb_set_position(self.running_counter.files);
        self.size_progress_span
            .pb_set_position(self.running_counter.bytes);
    }
}

pub fn default_spinner_style() -> ProgressStyle {
    ProgressStyle::with_template("{span_child_prefix}{spinner} {span_name}").unwrap()
}

/// Create a [`tracing::Span`] that will be displayed as a spinner in the terminal thanks to [`tracing_indicatif`].
#[macro_export]
macro_rules! default_spinner_span {
    ($name:expr) => {{
        let span = tracing::info_span!($name);
        tracing_indicatif::span_ext::IndicatifSpanExt::pb_set_style(
            &span,
            &$crate::progress::default_spinner_style(),
        );
        span.entered()
    }};
}
