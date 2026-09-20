//! Pluggable backup connectors (FTP, SFTP, Nitrado).

pub mod factory;
pub mod ftp;
pub mod matcher;
pub mod nitrado;
pub mod sftp;

use std::io::{Read, Write};
use std::time::SystemTime;

use anyhow::Result;

pub use factory::new_connector;

/// Metadata about a remote file.
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// Path relative to the connector's remote root.
    pub path: String,
    /// File size in bytes.
    pub size: i64,
    /// Last modification time.
    pub mod_time: SystemTime,
    /// Whether the entry is a directory.
    pub is_dir: bool,
}

/// Common configuration shared by all connectors.
#[derive(Debug, Clone, Default)]
pub struct Config {
    pub connector_type: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub key_file: String,
    pub passive: bool,
    pub tls: bool,
    pub api_key: String,
    pub service_id: String,
    pub remote_path: String,
    pub include: Vec<String>,
    pub exclude: Vec<String>,

    /// Retry settings.
    pub retry_attempts: i32,
    pub retry_delay: i32,
    pub retry_backoff: bool,
}

/// Interface implemented by every connector.
pub trait Connector {
    /// Establishes a connection to the remote server.
    fn connect(&mut self) -> Result<()>;

    /// Returns files matching the configured patterns under `remote_path`.
    fn list(&mut self) -> Result<Vec<FileInfo>>;

    /// Downloads a file, writing it to the provided writer.
    fn download(&mut self, remote_path: &str, w: &mut dyn Write) -> Result<()>;

    /// Uploads data from the reader to the remote path.
    fn upload(&mut self, r: &mut dyn Read, remote_path: &str) -> Result<()>;

    /// Terminates the connection.
    fn close(&mut self) -> Result<()>;

    /// Returns a human-readable name used for logging.
    fn name(&self) -> String;
}

/// Joins two POSIX-style path fragments.
pub(crate) fn join_posix(base: &str, name: &str) -> String {
    if base.is_empty() {
        return name.to_string();
    }
    if name.is_empty() {
        return base.to_string();
    }
    if base.ends_with('/') {
        format!("{base}{name}")
    } else {
        format!("{base}/{name}")
    }
}

/// Makes `full` relative to `remote_root`, stripping any leading separator.
pub(crate) fn relative_path(remote_root: &str, full: &str) -> String {
    if remote_root.is_empty() {
        return full.trim_start_matches('/').to_string();
    }
    let trimmed = remote_root.trim_end_matches('/');
    match full.strip_prefix(trimmed) {
        Some(rest) => rest.trim_start_matches('/').to_string(),
        None => full.trim_start_matches('/').to_string(),
    }
}
