//! Backup orchestration for a single server.

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};

use crate::backup::archive::{create_archive, timestamped_filename};
use crate::connector::Connector;
use crate::progress::Reporter;

/// Summary of a backup run.
#[derive(Debug, Default, Clone, Copy)]
pub struct Stats {
    /// Number of files downloaded.
    pub files: usize,
    /// Total number of bytes downloaded.
    pub bytes: i64,
    /// Wall-clock duration of the backup.
    pub duration: Duration,
}

/// Coordinates backup operations for a single server.
pub struct Manager {
    pub temp_dir: String,
    pub backup_location: String,
    pub progress: Reporter,
}

impl Manager {
    /// Pulls files via the connector, archives them and writes the archive to
    /// the backup location. Returns the archive path and run statistics.
    pub fn backup(&mut self, conn: &mut dyn Connector) -> Result<(String, Stats)> {
        let start = Instant::now();
        let mut stats = Stats::default();

        if self.backup_location.is_empty() {
            bail!("backup location is required");
        }

        let temp_dir = if self.temp_dir.is_empty() {
            Path::new(&self.backup_location)
                .join(".tmp")
                .to_string_lossy()
                .into_owned()
        } else {
            self.temp_dir.clone()
        };

        fs::create_dir_all(&temp_dir).with_context(|| format!("create temp dir {temp_dir}"))?;

        conn.connect().context("connect")?;

        let files = conn.list().context("list")?;

        let mut total_size = 0i64;
        for file in &files {
            if !file.is_dir {
                stats.files += 1;
                stats.bytes += file.size;
                total_size += file.size;
            }
        }

        self.progress.start(total_size, files.len());

        for file in &files {
            let local_path = Path::new(&temp_dir).join(&file.path);

            if file.is_dir {
                fs::create_dir_all(&local_path)
                    .with_context(|| format!("mkdir for {}", file.path))?;
                continue;
            }

            if let Some(parent) = local_path.parent() {
                fs::create_dir_all(parent).with_context(|| format!("mkdir for {}", file.path))?;
            }

            let mut output =
                File::create(&local_path).with_context(|| format!("create {}", file.path))?;

            self.progress.file_start(&file.path, file.size);

            {
                let progress = &mut self.progress;
                let path = file.path.clone();
                let size = file.size;
                let mut callback = |written: i64| progress.file_progress(&path, written, size);
                let mut writer = ProgressWriter {
                    inner: &mut output,
                    written: 0,
                    callback: &mut callback,
                };
                conn.download(&path, &mut writer)
                    .with_context(|| format!("download {}", file.path))?;
            }

            self.progress.file_done(&file.path);
        }

        self.progress.close();

        fs::create_dir_all(&self.backup_location)
            .with_context(|| format!("create backup dir {}", self.backup_location))?;

        let archive_path = Path::new(&self.backup_location).join(timestamped_filename());
        create_archive(Path::new(&temp_dir), &archive_path).context("create archive")?;

        stats.duration = start.elapsed();
        Ok((archive_path.to_string_lossy().into_owned(), stats))
    }
}

/// Wraps a writer to report incremental bytes written.
struct ProgressWriter<'a> {
    inner: &'a mut dyn Write,
    written: i64,
    callback: &'a mut dyn FnMut(i64),
}

impl Write for ProgressWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.inner.write(buf)?;
        self.written += n as i64;
        (self.callback)(self.written);
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connector::{Config, FileInfo};
    use std::time::SystemTime;

    struct MockConnector {
        files: Vec<FileInfo>,
        data: std::collections::HashMap<String, String>,
        connected: bool,
    }

    impl Connector for MockConnector {
        fn connect(&mut self) -> Result<()> {
            self.connected = true;
            Ok(())
        }

        fn list(&mut self) -> Result<Vec<FileInfo>> {
            Ok(self.files.clone())
        }

        fn download(&mut self, remote_path: &str, w: &mut dyn Write) -> Result<()> {
            if !self.connected {
                bail!("not connected");
            }
            let data = self.data.get(remote_path).cloned().unwrap_or_default();
            w.write_all(data.as_bytes())?;
            Ok(())
        }

        fn upload(&mut self, _r: &mut dyn io::Read, _remote_path: &str) -> Result<()> {
            Ok(())
        }

        fn close(&mut self) -> Result<()> {
            self.connected = false;
            Ok(())
        }

        fn name(&self) -> String {
            "mock".to_string()
        }
    }

    fn mock() -> MockConnector {
        let mut data = std::collections::HashMap::new();
        data.insert("file1.txt".to_string(), "hello".to_string());
        data.insert("nested/file2.txt".to_string(), "world".to_string());

        MockConnector {
            files: vec![
                FileInfo {
                    path: "file1.txt".to_string(),
                    size: 5,
                    mod_time: SystemTime::now(),
                    is_dir: false,
                },
                FileInfo {
                    path: "nested/file2.txt".to_string(),
                    size: 5,
                    mod_time: SystemTime::now(),
                    is_dir: false,
                },
            ],
            data,
            connected: false,
        }
    }

    #[test]
    fn backup_creates_archive_and_stats() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut conn = mock();

        let mut manager = Manager {
            temp_dir: String::new(),
            backup_location: tmp.path().to_string_lossy().into_owned(),
            progress: crate::progress::new(&crate::log::Logger::new(), "json"),
        };

        let (archive_path, stats) = manager.backup(&mut conn).expect("backup");

        assert!(Path::new(&archive_path).exists(), "archive not created");
        assert_eq!(Path::new(&archive_path).parent().unwrap(), tmp.path());
        assert_eq!(stats.files, 2);
        assert_eq!(stats.bytes, ("hello".len() + "world".len()) as i64);
    }

    #[test]
    fn backup_requires_location() {
        let mut conn = mock();
        let mut manager = Manager {
            temp_dir: String::new(),
            backup_location: String::new(),
            progress: crate::progress::new(&crate::log::Logger::new(), "json"),
        };
        assert!(manager.backup(&mut conn).is_err());
    }

    #[test]
    fn connector_config_defaults() {
        let cfg = Config::default();
        assert_eq!(cfg.retry_attempts, 0);
    }
}
