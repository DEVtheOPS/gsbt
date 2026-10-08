//! Structured logging with markup support.
//!
//! The logger supports lightweight markup tags such as `[bold]` / `[/bold]`
//! which are stripped in both text and JSON output. Two output formats are
//! supported: `text` (default) and `json`.

use std::collections::BTreeMap;
use std::io::{self, Write};
use std::sync::{Arc, LazyLock, Mutex};

use chrono::{SecondsFormat, Utc};
use regex::Regex;
use serde_json::Value;

/// Log level, ordered from most to least verbose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Debug,
    Info,
    Warn,
    Error,
}

impl Level {
    /// Lower-case string representation used in JSON output.
    pub fn as_str(self) -> &'static str {
        match self {
            Level::Debug => "debug",
            Level::Info => "info",
            Level::Warn => "warn",
            Level::Error => "error",
        }
    }
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Structured metadata attached to a log entry.
pub type Meta = BTreeMap<String, Value>;

/// Build a [`Meta`] map conveniently: `meta! { "files" => 3 }`.
#[macro_export]
macro_rules! meta {
    ($($key:expr => $value:expr),* $(,)?) => {{
        let mut map = std::collections::BTreeMap::new();
        $(map.insert($key.to_string(), serde_json::json!($value));)*
        map
    }};
}

/// Output format for the logger.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
}

impl OutputFormat {
    /// Parse an output format from its CLI string value.
    pub fn parse(value: &str) -> Self {
        match value {
            "json" => OutputFormat::Json,
            _ => OutputFormat::Text,
        }
    }
}

static MARKUP_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[(/?)([a-z]+)\]").expect("valid markup regex"));

/// Removes markup tags, e.g. `[bold]text[/bold]` becomes `text`.
pub fn strip_markup(s: &str) -> String {
    MARKUP_REGEX.replace_all(s, "").into_owned()
}

/// Shared, cloneable writer used to capture or forward log output.
///
/// Useful for tests and for capturing output from multiple threads.
#[derive(Clone, Default)]
pub struct BufferWriter(Arc<Mutex<Vec<u8>>>);

impl BufferWriter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the captured bytes as a lossy UTF-8 string.
    pub fn contents(&self) -> String {
        String::from_utf8_lossy(&self.0.lock().expect("buffer lock")).into_owned()
    }

    /// Clears the captured bytes.
    pub fn clear(&self) {
        self.0.lock().expect("buffer lock").clear();
    }
}

impl Write for BufferWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.lock().expect("buffer lock").extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Structured logger with markup support.
///
/// Clones share the underlying output writers but carry their own
/// level/prefix/format configuration.
#[derive(Clone)]
pub struct Logger {
    out: Arc<Mutex<Box<dyn Write + Send>>>,
    err: Arc<Mutex<Box<dyn Write + Send>>>,
    level: Level,
    prefix: String,
    format: OutputFormat,
    quiet: bool,
    verbose: bool,
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

impl Logger {
    /// Creates a logger writing to stdout/stderr.
    pub fn new() -> Self {
        Self::with_writers(Box::new(io::stdout()), Box::new(io::stderr()))
    }

    /// Creates a logger with custom writers (useful for testing).
    pub fn with_writers(out: Box<dyn Write + Send>, err: Box<dyn Write + Send>) -> Self {
        Self {
            out: Arc::new(Mutex::new(out)),
            err: Arc::new(Mutex::new(err)),
            level: Level::Info,
            prefix: String::new(),
            format: OutputFormat::Text,
            quiet: false,
            verbose: false,
        }
    }

    /// Sets the output writer for non-error logs.
    pub fn set_output(&mut self, w: Box<dyn Write + Send>) {
        self.out = Arc::new(Mutex::new(w));
    }

    /// Sets the output writer for error logs.
    pub fn set_error_output(&mut self, w: Box<dyn Write + Send>) {
        self.err = Arc::new(Mutex::new(w));
    }

    /// Sets the output format.
    pub fn set_output_format(&mut self, format: &str) {
        self.format = OutputFormat::parse(format);
    }

    /// Enables quiet mode (errors only).
    pub fn set_quiet(&mut self, quiet: bool) {
        self.quiet = quiet;
        if quiet {
            self.level = Level::Error;
        }
    }

    /// Enables verbose mode (debug + metadata).
    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
        if verbose && self.level > Level::Debug {
            self.level = Level::Debug;
        }
    }

    /// Sets the minimum log level.
    pub fn set_level(&mut self, level: Level) {
        self.level = level;
    }

    /// Returns a logger with the given prefix, keeping the same writers/config.
    pub fn with_prefix(&self, prefix: impl Into<String>) -> Self {
        let mut logger = self.clone();
        logger.prefix = prefix.into();
        logger
    }

    /// Returns `true` if quiet mode is enabled.
    pub fn is_quiet(&self) -> bool {
        self.quiet
    }

    /// Returns `true` if verbose mode is enabled.
    pub fn is_verbose(&self) -> bool {
        self.verbose
    }

    /// Logs a debug-level message.
    pub fn debug(&self, msg: impl AsRef<str>) {
        self.log(Level::Debug, msg.as_ref(), None);
    }

    /// Logs an info-level message.
    pub fn info(&self, msg: impl AsRef<str>) {
        self.log(Level::Info, msg.as_ref(), None);
    }

    /// Logs a warning-level message.
    pub fn warn(&self, msg: impl AsRef<str>) {
        self.log(Level::Warn, msg.as_ref(), None);
    }

    /// Logs an error-level message.
    pub fn error(&self, msg: impl AsRef<str>) {
        self.log(Level::Error, msg.as_ref(), None);
    }

    /// Logs an info-level message with structured metadata.
    pub fn info_with(&self, msg: impl AsRef<str>, meta: Meta) {
        self.log(Level::Info, msg.as_ref(), Some(meta));
    }

    /// Logs an error-level message with structured metadata.
    pub fn error_with(&self, msg: impl AsRef<str>, meta: Meta) {
        self.log(Level::Error, msg.as_ref(), Some(meta));
    }

    fn log(&self, level: Level, msg: &str, meta: Option<Meta>) {
        if level < self.level {
            return;
        }

        let writer = if level >= Level::Error {
            &self.err
        } else {
            &self.out
        };
        let mut guard = writer.lock().expect("log writer lock");

        match self.format {
            OutputFormat::Json => write_json(&mut *guard, level, msg, &self.prefix, meta.as_ref()),
            OutputFormat::Text => write_text(
                &mut *guard,
                level,
                msg,
                &self.prefix,
                meta.as_ref(),
                self.verbose,
            ),
        }
    }
}

fn write_json(w: &mut dyn Write, level: Level, msg: &str, prefix: &str, meta: Option<&Meta>) {
    let mut entry = serde_json::Map::new();
    entry.insert(
        "timestamp".into(),
        Value::String(Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)),
    );
    entry.insert("level".into(), Value::String(level.as_str().into()));
    entry.insert("message".into(), Value::String(strip_markup(msg)));

    if !prefix.is_empty() {
        entry.insert("prefix".into(), Value::String(strip_markup(prefix)));
    }

    if let Some(meta) = meta {
        if !meta.is_empty() {
            entry.insert(
                "metadata".into(),
                Value::Object(meta.clone().into_iter().collect()),
            );
        }
    }

    let _ = writeln!(w, "{}", Value::Object(entry));
}

fn write_text(
    w: &mut dyn Write,
    _level: Level,
    msg: &str,
    prefix: &str,
    meta: Option<&Meta>,
    verbose: bool,
) {
    let plain = strip_markup(msg);

    if prefix.is_empty() {
        let _ = writeln!(w, "{plain}");
    } else {
        let _ = writeln!(w, "{} {}", strip_markup(prefix), plain);
    }

    if verbose {
        if let Some(meta) = meta {
            if !meta.is_empty() {
                let _ = writeln!(w, "  {}", format_metadata(meta));
            }
        }
    }
}

/// Formats metadata as space-separated `key=value` pairs.
pub fn format_metadata(meta: &Meta) -> String {
    meta.iter()
        .map(|(k, v)| match v {
            Value::String(s) => format!("{k}={s}"),
            other => format!("{k}={other}"),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn logger() -> (Logger, BufferWriter, BufferWriter) {
        let out = BufferWriter::new();
        let err = BufferWriter::new();
        let logger = Logger::with_writers(Box::new(out.clone()), Box::new(err.clone()));
        (logger, out, err)
    }

    #[test]
    fn text_mode_strips_markup() {
        let (mut logger, out, _) = logger();
        logger.set_output_format("text");

        logger.info("plain message");
        assert!(out.contents().contains("plain message"));

        out.clear();
        logger.info("[bold]formatted[/bold] message");
        assert!(!out.contents().contains("[bold]"));
        assert!(out.contents().contains("formatted message"));
    }

    #[test]
    fn json_mode() {
        let (mut logger, out, _) = logger();
        logger.set_output_format("json");

        logger.info_with("[bold]test message[/bold]", meta! { "key" => "value" });

        let entry: Value = serde_json::from_str(out.contents().trim()).expect("valid json");
        assert_eq!(entry["message"], "test message");
        assert_eq!(entry["level"], "info");
        assert_eq!(entry["metadata"]["key"], "value");
    }

    #[test]
    fn with_prefix() {
        let (mut logger, out, _) = logger();
        logger.set_output_format("text");

        logger.with_prefix("[test-server]").info("message");

        assert!(out.contents().contains("[test-server]"));
        assert!(out.contents().contains("message"));
    }

    #[test]
    fn with_prefix_markup_stripped() {
        let (mut logger, out, _) = logger();

        logger.set_output_format("text");
        logger
            .with_prefix("[bold][cyan]server[/cyan][/bold]")
            .info("[green]message[/green]");

        let output = out.contents();
        assert!(!output.contains("[bold]"));
        assert!(!output.contains("[cyan]"));
        assert!(output.contains("server"));

        out.clear();
        logger.set_output_format("json");
        logger
            .with_prefix("[bold][cyan]server[/cyan][/bold]")
            .info("message");

        let entry: Value = serde_json::from_str(out.contents().trim()).expect("valid json");
        let prefix = entry["prefix"].as_str().unwrap_or_default();
        assert!(!prefix.contains('['));
    }

    #[test]
    fn quiet_mode() {
        let (mut logger, out, err) = logger();
        logger.set_quiet(true);

        logger.info("info message");
        assert_eq!(out.contents(), "");

        logger.error("error message");
        assert!(err.contents().contains("error message"));
    }

    #[test]
    fn verbose_mode_shows_debug() {
        let (mut logger, out, _) = logger();
        logger.set_output_format("text");
        logger.set_verbose(true);

        logger.debug("debug message");
        assert!(out.contents().contains("debug message"));
    }

    #[test]
    fn metadata_in_verbose_text() {
        let (mut logger, out, _) = logger();
        logger.set_output_format("text");
        logger.set_verbose(true);

        logger.info_with("test", meta! { "foo" => "bar", "count" => 42 });

        let output = out.contents();
        assert!(output.contains("foo=bar"));
        assert!(output.contains("count=42"));
    }

    #[test]
    fn metadata_hidden_without_verbose() {
        let (mut logger, out, _) = logger();
        logger.set_output_format("text");
        logger.set_verbose(false);

        logger.info_with("test", meta! { "foo" => "bar" });
        assert!(!out.contents().contains("foo=bar"));
    }

    #[test]
    fn filtering_by_level() {
        let (mut logger, out, err) = logger();
        logger.set_output_format("text");
        logger.set_level(Level::Warn);

        logger.debug("debug");
        logger.info("info");
        logger.warn("warn");
        logger.error("error");

        let output = out.contents();
        assert!(!output.contains("debug"));
        assert!(!output.contains("info"));
        assert!(output.contains("warn"));
        assert!(err.contents().contains("error"));
    }

    #[test]
    fn strip_markup_cases() {
        assert_eq!(strip_markup("plain text"), "plain text");
        assert_eq!(strip_markup("[bold]bold text[/bold]"), "bold text");
        assert_eq!(strip_markup("[green]colored[/green] text"), "colored text");
        assert_eq!(strip_markup("[bold][red]nested[/red][/bold]"), "nested");
    }
}
