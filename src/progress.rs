//! Progress reporting for long-running operations.
//!
//! Three reporters are available:
//! - [`Reporter::Null`] - no output (quiet/JSON modes).
//! - [`Reporter::Simple`] - one informational line per event (piped output).
//! - [`Reporter::Bar`] - an `indicatif` progress bar, one per server, used
//!   when the terminal supports it.

use std::io::{self, Write};
use std::time::Duration;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use crate::log::Logger;

/// Reports progress for downloads and uploads.
pub enum Reporter {
    /// No-op reporter used for quiet and JSON output modes.
    Null(NullProgress),
    /// Text-based reporter used when output is not a terminal.
    Simple(SimpleProgress),
    /// `indicatif` progress bar used when the terminal supports it.
    Bar(BarProgress),
}

impl Reporter {
    /// Initialises progress tracking with total bytes and file count.
    pub fn start(&mut self, total_bytes: i64, file_count: usize) {
        match self {
            Reporter::Null(p) => p.start(total_bytes, file_count),
            Reporter::Simple(p) => p.start(total_bytes, file_count),
            Reporter::Bar(p) => p.start(total_bytes, file_count),
        }
    }

    /// Marks the beginning of a file download/upload.
    pub fn file_start(&mut self, name: &str, size: i64) {
        match self {
            Reporter::Null(p) => p.file_start(name, size),
            Reporter::Simple(p) => p.file_start(name, size),
            Reporter::Bar(p) => p.file_start(name, size),
        }
    }

    /// Updates progress for the current file.
    pub fn file_progress(&mut self, name: &str, written: i64, size: i64) {
        match self {
            Reporter::Null(p) => p.file_progress(name, written, size),
            Reporter::Simple(p) => p.file_progress(name, written, size),
            Reporter::Bar(p) => p.file_progress(name, written, size),
        }
    }

    /// Marks completion of the current file.
    pub fn file_done(&mut self, name: &str) {
        match self {
            Reporter::Null(p) => p.file_done(name),
            Reporter::Simple(p) => p.file_done(name),
            Reporter::Bar(p) => p.file_done(name),
        }
    }

    /// Logs an informational message.
    pub fn message(&mut self, msg: &str) {
        match self {
            Reporter::Null(p) => p.message(msg),
            Reporter::Simple(p) => p.message(msg),
            Reporter::Bar(p) => p.message(msg),
        }
    }

    /// Cleans up resources.
    pub fn close(&mut self) {
        match self {
            Reporter::Null(p) => p.close(),
            Reporter::Simple(p) => p.close(),
            Reporter::Bar(p) => p.close(),
        }
    }
}

/// Creates a simple reporter based on the output format (no bars).
///
/// Prefer [`ProgressFactory`] in the CLI, which enables bars when the terminal
/// supports them.
pub fn new(logger: &Logger, format: &str) -> Reporter {
    if logger.is_quiet() || format == "json" {
        Reporter::Null(NullProgress)
    } else {
        Reporter::Simple(SimpleProgress::new(logger.clone()))
    }
}

/// Creates per-server reporters, sharing a single multi-progress when enabled.
pub struct ProgressFactory {
    multi: Option<MultiProgress>,
    format: String,
}

impl ProgressFactory {
    /// Creates a factory. Bars are only enabled for text output on a terminal
    /// when not in quiet mode.
    pub fn new(format: &str, quiet: bool, terminal: bool) -> Self {
        let enabled = format == "text" && !quiet && terminal;
        Self {
            multi: enabled.then(MultiProgress::new),
            format: format.to_string(),
        }
    }

    /// Returns `true` when bars are enabled.
    pub fn is_enabled(&self) -> bool {
        self.multi.is_some()
    }

    /// Creates a reporter for a single server.
    pub fn reporter(&self, logger: &Logger, server: &str) -> Reporter {
        match &self.multi {
            Some(multi) => Reporter::Bar(BarProgress::new(multi, server)),
            None => new(logger, &self.format),
        }
    }

    /// Wraps the log writers so output is printed above the bars instead of
    /// being overwritten by them.
    pub fn wrap_writers(
        &self,
        out: Box<dyn Write + Send>,
        err: Box<dyn Write + Send>,
    ) -> (Box<dyn Write + Send>, Box<dyn Write + Send>) {
        match &self.multi {
            Some(multi) => (
                Box::new(MultiProgressWriter::new(multi.clone())),
                Box::new(MultiProgressWriter::new(multi.clone())),
            ),
            None => (out, err),
        }
    }

    /// Clears any remaining bars once the run has finished.
    pub fn finish(&self) {
        if let Some(multi) = &self.multi {
            let _ = multi.clear();
        }
    }
}

/// A writer that forwards whole lines to a [`MultiProgress`] so log output
/// does not corrupt the bars.
#[derive(Clone)]
struct MultiProgressWriter {
    multi: MultiProgress,
}

impl MultiProgressWriter {
    fn new(multi: MultiProgress) -> Self {
        Self { multi }
    }
}

impl Write for MultiProgressWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let text = String::from_utf8_lossy(buf);
        for line in text.split_inclusive('\n') {
            let line = line.trim_end_matches(['\n', '\r']);
            if !line.is_empty() {
                let _ = self.multi.println(line);
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// No-op reporter for quiet/JSON modes.
pub struct NullProgress;

impl NullProgress {
    pub fn start(&self, _total_bytes: i64, _file_count: usize) {}
    pub fn file_start(&self, _name: &str, _size: i64) {}
    pub fn file_progress(&self, _name: &str, _written: i64, _size: i64) {}
    pub fn file_done(&self, _name: &str) {}
    pub fn message(&self, _msg: &str) {}
    pub fn close(&self) {}
}

/// Text-based progress reporter.
pub struct SimpleProgress {
    logger: Logger,
}

impl SimpleProgress {
    fn new(logger: Logger) -> Self {
        Self { logger }
    }

    pub fn start(&self, total_bytes: i64, file_count: usize) {
        self.logger.info(format!(
            "Files: {file_count}, Total: {:.1} MB",
            mega(total_bytes)
        ));
    }

    pub fn file_start(&self, name: &str, size: i64) {
        self.logger.info(format!("- {name} ({:.1} MB)", mega(size)));
    }

    pub fn file_progress(&self, _name: &str, _written: i64, _size: i64) {
        // Don't spam output; only start/done are shown.
    }

    pub fn file_done(&self, name: &str) {
        self.logger.debug(format!("  done {name}"));
    }

    pub fn message(&self, msg: &str) {
        self.logger.info(msg);
    }

    pub fn close(&self) {}
}

/// `indicatif`-based progress bar, one per server.
pub struct BarProgress {
    bar: ProgressBar,
    total_bytes: u64,
    completed_bytes: u64,
    current_file_size: u64,
    total_files: usize,
    files_done: usize,
}

impl BarProgress {
    fn new(multi: &MultiProgress, server: &str) -> Self {
        let bar = multi.add(ProgressBar::new(0));
        bar.set_style(bar_style());
        bar.set_prefix(truncate(server, 20));
        bar.enable_steady_tick(Duration::from_millis(100));
        Self {
            bar,
            total_bytes: 0,
            completed_bytes: 0,
            current_file_size: 0,
            total_files: 0,
            files_done: 0,
        }
    }

    pub fn start(&mut self, total_bytes: i64, file_count: usize) {
        self.total_bytes = total_bytes.max(0) as u64;
        self.total_files = file_count;
        self.bar.set_length(self.total_bytes);
        self.set_files_message();
    }

    pub fn file_start(&mut self, name: &str, size: i64) {
        self.current_file_size = size.max(0) as u64;
        self.bar.set_message(format!(
            "[{}/{}] {}",
            (self.files_done + 1).min(self.total_files.max(1)),
            self.total_files.max(1),
            truncate(name, 48),
        ));
    }

    pub fn file_progress(&mut self, _name: &str, written: i64, _size: i64) {
        let position = self.completed_bytes.saturating_add(written.max(0) as u64);
        self.bar.set_position(position.min(self.total_bytes));
    }

    pub fn file_done(&mut self, _name: &str) {
        self.completed_bytes = self.completed_bytes.saturating_add(self.current_file_size);
        self.files_done += 1;
        self.bar
            .set_position(self.completed_bytes.min(self.total_bytes));
        self.set_files_message();
    }

    pub fn message(&mut self, msg: &str) {
        self.bar.set_message(truncate(msg, 48));
    }

    pub fn close(&mut self) {
        self.bar.set_position(self.total_bytes);
        self.bar.finish_and_clear();
    }

    fn set_files_message(&self) {
        self.bar
            .set_message(format!("{}/{} files", self.files_done, self.total_files));
    }
}

fn bar_style() -> ProgressStyle {
    ProgressStyle::with_template(
        "{spinner:.green} {prefix} [{bar:32.cyan/blue}] {bytes}/{total_bytes} ({percent}%) {wide_msg}",
    )
    .unwrap_or_else(|_| ProgressStyle::default_bar())
    .progress_chars("=> ")
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let prefix: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{prefix}…")
    }
}

fn mega(bytes: i64) -> f64 {
    bytes as f64 / 1e6
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::{BufferWriter, Logger};

    fn logger(quiet: bool) -> (Logger, BufferWriter) {
        let out = BufferWriter::new();
        let mut logger = Logger::with_writers(Box::new(out.clone()), Box::new(out.clone()));
        logger.set_quiet(quiet);
        (logger, out)
    }

    #[test]
    fn null_progress_emits_nothing() {
        let (logger, out) = logger(true);
        let mut p = new(&logger, "text");

        p.start(1000, 5);
        p.file_start("test.txt", 100);
        p.file_progress("test.txt", 50, 100);
        p.file_done("test.txt");
        p.message("test");
        p.close();

        assert_eq!(out.contents(), "");
    }

    #[test]
    fn simple_progress_outputs() {
        let (logger, out) = logger(false);
        let mut p = new(&logger, "text");

        p.start(1000, 5);
        assert!(out.contents().contains("Files: 5"));

        out.clear();
        p.file_start("test.txt", 100);
        assert!(out.contents().contains("test.txt"));

        p.file_done("test.txt");
        p.close();
    }

    #[test]
    fn simple_progress_message() {
        let (logger, out) = logger(false);
        let mut p = new(&logger, "text");
        p.message("custom message");
        assert!(out.contents().contains("custom message"));
    }

    #[test]
    fn new_returns_expected_variant() {
        let (mut logger, _) = logger(false);

        logger.set_quiet(true);
        assert!(matches!(new(&logger, "text"), Reporter::Null(_)));

        logger.set_quiet(false);
        assert!(matches!(new(&logger, "json"), Reporter::Null(_)));

        assert!(matches!(new(&logger, "text"), Reporter::Simple(_)));
    }

    #[test]
    fn factory_enables_bars_only_for_interactive_text() {
        assert!(!ProgressFactory::new("text", false, false).is_enabled());
        assert!(!ProgressFactory::new("text", true, true).is_enabled());
        assert!(!ProgressFactory::new("json", false, true).is_enabled());
        assert!(ProgressFactory::new("text", false, true).is_enabled());
    }

    #[test]
    fn factory_reports_bars_per_server() {
        let factory = ProgressFactory::new("text", false, true);
        let (logger, _) = logger(false);
        assert!(matches!(
            factory.reporter(&logger, "server-a"),
            Reporter::Bar(_)
        ));
        assert!(matches!(
            factory.reporter(&logger, "server-b"),
            Reporter::Bar(_)
        ));
    }

    #[test]
    fn factory_falls_back_to_simple_when_disabled() {
        let factory = ProgressFactory::new("text", false, false);
        let (logger, _) = logger(false);
        assert!(matches!(
            factory.reporter(&logger, "server-a"),
            Reporter::Simple(_)
        ));
    }

    #[test]
    fn truncate_shortens_long_names() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("abcdefgh", 5), "abcd…");
    }
}
