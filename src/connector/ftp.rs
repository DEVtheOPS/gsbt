//! FTP connector (with optional explicit TLS).

use std::io::{Read, Write};
use std::net::ToSocketAddrs;
use std::time::Duration;

use anyhow::{bail, Result};
use suppaftp::list::ListParser;
use suppaftp::{Mode, NativeTlsFtpStream};

use super::matcher::matches_patterns;
use super::{join_posix, relative_path, Config, Connector, FileInfo};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

/// FTP connector implementation.
///
/// The stream is always created as a TLS-capable stream; the TLS handshake is
/// only performed when explicit TLS is requested.
pub struct FtpConnector {
    config: Config,
    stream: Option<NativeTlsFtpStream>,
}

impl FtpConnector {
    /// Creates a new FTP connector, defaulting the port to 21.
    pub fn new(mut cfg: Config) -> Self {
        if cfg.port == 0 {
            cfg.port = 21;
        }
        Self {
            config: cfg,
            stream: None,
        }
    }

    /// Builds a new FTP connector from an existing configuration (used by the
    /// Nitrado connector, which delegates to FTP).
    pub fn from_config(cfg: Config) -> Self {
        Self::new(cfg)
    }

    fn stream_mut(&mut self) -> Result<&mut NativeTlsFtpStream> {
        match self.stream.as_mut() {
            Some(stream) => Ok(stream),
            None => bail!("not connected"),
        }
    }

    fn walk_dir(&mut self, dir: &str, files: &mut Vec<FileInfo>) -> Result<()> {
        let entries = self
            .stream_mut()?
            .list(Some(dir))
            .map_err(|e| anyhow::anyhow!("failed to list {dir}: {e}"))?;

        for line in entries {
            let entry = match ListParser::parse_posix(&line) {
                Ok(entry) => entry,
                Err(_) => continue,
            };

            let name = entry.name();
            if name == "." || name == ".." {
                continue;
            }

            let full_path = join_posix(dir, name);
            let rel_path = relative_path(&self.config.remote_path, &full_path);
            let is_dir = entry.is_directory();

            files.push(FileInfo {
                path: rel_path,
                size: entry.size() as i64,
                mod_time: entry.modified(),
                is_dir,
            });

            if is_dir {
                self.walk_dir(&full_path, files)?;
            }
        }

        Ok(())
    }
}

impl Connector for FtpConnector {
    fn name(&self) -> String {
        format!("ftp://{}:{}", self.config.host, self.config.port)
    }

    fn connect(&mut self) -> Result<()> {
        let addr = format!("{}:{}", self.config.host, self.config.port)
            .to_socket_addrs()
            .map_err(|e| anyhow::anyhow!("failed to resolve FTP address: {e}"))?
            .next()
            .ok_or_else(|| anyhow::anyhow!("failed to resolve FTP address"))?;

        let mut stream = NativeTlsFtpStream::connect_timeout(addr, CONNECT_TIMEOUT)
            .map_err(|e| anyhow::anyhow!("failed to connect to FTP: {e}"))?;

        if self.config.tls {
            let connector = suppaftp::native_tls::TlsConnector::new()
                .map_err(|e| anyhow::anyhow!("failed to initialize TLS: {e}"))?;
            let connector = suppaftp::NativeTlsConnector::from(connector);
            stream = stream
                .into_secure(connector, &self.config.host)
                .map_err(|e| anyhow::anyhow!("failed to establish TLS: {e}"))?;
        }

        stream.set_mode(if self.config.passive {
            Mode::Passive
        } else {
            Mode::Active
        });

        stream
            .login(&self.config.username, &self.config.password)
            .map_err(|e| anyhow::anyhow!("FTP login failed: {e}"))?;

        self.stream = Some(stream);
        Ok(())
    }

    fn list(&mut self) -> Result<Vec<FileInfo>> {
        // Ensure we are connected before walking.
        self.stream_mut()?;

        let mut files = Vec::new();
        let root = self.config.remote_path.clone();
        self.walk_dir(&root, &mut files)?;

        Ok(files
            .into_iter()
            .filter(|file| {
                !file.is_dir
                    && matches_patterns(&file.path, &self.config.include, &self.config.exclude)
            })
            .collect())
    }

    fn download(&mut self, remote_path: &str, w: &mut dyn Write) -> Result<()> {
        let full_path = join_posix(&self.config.remote_path, remote_path);
        self.stream_mut()?
            .retr(&full_path, |reader| {
                std::io::copy(reader, &mut *w)
                    .map(|_| ())
                    .map_err(suppaftp::FtpError::ConnectionError)
            })
            .map_err(|e| anyhow::anyhow!("failed to download {remote_path}: {e}"))
    }

    fn upload(&mut self, r: &mut dyn Read, remote_path: &str) -> Result<()> {
        let full_path = join_posix(&self.config.remote_path, remote_path);
        let dir = match full_path.rsplit_once('/') {
            Some((dir, _)) => dir.to_string(),
            None => String::new(),
        };

        let stream = self.stream_mut()?;

        // Create intermediate directories recursively. `MKD` only creates a
        // single level, so nested archive paths need each parent first. Errors
        // are ignored because the directory may already exist.
        for prefix in super::dir_prefixes(&dir) {
            let _ = stream.mkdir(&prefix);
        }

        let mut data = stream
            .put_with_stream(&full_path)
            .map_err(|e| anyhow::anyhow!("failed to upload {remote_path}: {e}"))?;
        std::io::copy(r, &mut data)
            .map_err(|e| anyhow::anyhow!("failed to upload {remote_path}: {e}"))?;
        data.finish()
            .map_err(|e| anyhow::anyhow!("failed to upload {remote_path}: {e}"))?;
        Ok(())
    }

    fn close(&mut self) -> Result<()> {
        if let Some(mut stream) = self.stream.take() {
            let _ = stream.quit();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_includes_host_and_default_port() {
        let cfg = Config {
            connector_type: "ftp".to_string(),
            host: "localhost".to_string(),
            ..Default::default()
        };
        let conn = FtpConnector::new(cfg);
        assert_eq!(conn.name(), "ftp://localhost:21");
    }

    #[test]
    fn download_before_connect_errors() {
        let cfg = Config {
            connector_type: "ftp".to_string(),
            host: "localhost".to_string(),
            remote_path: "/saves/".to_string(),
            ..Default::default()
        };
        let mut conn = FtpConnector::new(cfg);
        let mut sink = Vec::new();
        assert!(conn.download("file.txt", &mut sink).is_err());
    }
}
