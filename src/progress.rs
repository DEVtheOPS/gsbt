//! Progress reporting for long-running operations.

use crate::log::Logger;

/// Reports progress for downloads and uploads.
pub enum Reporter {
    /// No-op reporter used for quiet and JSON output modes.
    Null(NullProgress),
    /// Text-based reporter used for the default output mode.
    Simple(SimpleProgress),
}

impl Reporter {
    /// Initialises progress tracking with total bytes and file count.
    pub fn start(&mut self, total_bytes: i64, file_count: usize) {
        match self {
            Reporter::Null(p) => p.start(total_bytes, file_count),
            Reporter::Simple(p) => p.start(total_bytes, file_count),
        }
    }

    /// Marks the beginning of a file download/upload.
    pub fn file_start(&mut self, name: &str, size: i64) {
        match self {
            Reporter::Null(p) => p.file_start(name, size),
            Reporter::Simple(p) => p.file_start(name, size),
        }
    }

    /// Updates progress for the current file.
    pub fn file_progress(&mut self, name: &str, written: i64, size: i64) {
        match self {
            Reporter::Null(p) => p.file_progress(name, written, size),
            Reporter::Simple(p) => p.file_progress(name, written, size),
        }
    }

    /// Marks completion of the current file.
    pub fn file_done(&mut self, name: &str) {
        match self {
            Reporter::Null(p) => p.file_done(name),
            Reporter::Simple(p) => p.file_done(name),
        }
    }

    /// Logs an informational message.
    pub fn message(&mut self, msg: &str) {
        match self {
            Reporter::Null(p) => p.message(msg),
            Reporter::Simple(p) => p.message(msg),
        }
    }

    /// Cleans up resources.
    pub fn close(&mut self) {
        match self {
            Reporter::Null(p) => p.close(),
            Reporter::Simple(p) => p.close(),
        }
    }
}

/// Creates a progress reporter based on the output format.
pub fn new(logger: &Logger, format: &str) -> Reporter {
    if logger.is_quiet() || format == "json" {
        Reporter::Null(NullProgress)
    } else {
        Reporter::Simple(SimpleProgress::new(logger.clone()))
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
}
