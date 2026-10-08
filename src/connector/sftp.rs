//! SFTP connector built on the synchronous `ssh2` crate.

use std::io::{self, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::Path;
use std::time::{Duration, UNIX_EPOCH};

use anyhow::{bail, Result};
use ssh2::{Session, Sftp};

use super::matcher::matches_patterns;
use super::{join_posix, relative_path, Config, Connector, FileInfo};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
const IO_TIMEOUT_MS: u32 = 30_000;

/// SFTP connector implementation.
pub struct SftpConnector {
    config: Config,
    session: Option<Session>,
    sftp: Option<Sftp>,
}

impl SftpConnector {
    /// Creates a new SFTP connector, defaulting the port to 22.
    pub fn new(mut cfg: Config) -> Self {
        if cfg.port == 0 {
            cfg.port = 22;
        }
        Self {
            config: cfg,
            session: None,
            sftp: None,
        }
    }

    fn walk_dir(&mut self, dir: &str, files: &mut Vec<FileInfo>) -> Result<()> {
        let entries = {
            let sftp = match self.sftp.as_ref() {
                Some(sftp) => sftp,
                None => bail!("not connected"),
            };
            sftp.readdir(Path::new(dir))
                .map_err(|e| anyhow::anyhow!("failed to list {dir}: {e}"))?
        };

        for (path, stat) in entries {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            if name == "." || name == ".." {
                continue;
            }

            let full_path = join_posix(dir, &name);
            let rel_path = relative_path(&self.config.remote_path, &full_path);
            let is_dir = stat.is_dir();
            let mod_time = stat
                .mtime
                .map(|secs| UNIX_EPOCH + Duration::from_secs(secs))
                .unwrap_or(UNIX_EPOCH);

            files.push(FileInfo {
                path: rel_path,
                size: stat.size.unwrap_or(0) as i64,
                mod_time,
                is_dir,
            });

            if is_dir {
                self.walk_dir(&full_path, files)?;
            }
        }

        Ok(())
    }

    fn ensure_remote_dir(&self, dir: &str) -> Result<()> {
        let sftp = match self.sftp.as_ref() {
            Some(sftp) => sftp,
            None => bail!("not connected"),
        };

        for prefix in super::dir_prefixes(dir) {
            // Ignore errors: the directory may already exist.
            let _ = sftp.mkdir(Path::new(&prefix), 0o755);
        }

        Ok(())
    }
}

impl Connector for SftpConnector {
    fn name(&self) -> String {
        format!("sftp://{}:{}", self.config.host, self.config.port)
    }

    fn connect(&mut self) -> Result<()> {
        if self.config.key_file.is_empty() && self.config.password.is_empty() {
            bail!("no authentication method provided (need password or key_file)");
        }

        let addr = format!("{}:{}", self.config.host, self.config.port)
            .to_socket_addrs()
            .map_err(|e| anyhow::anyhow!("failed to resolve SSH address: {e}"))?
            .next()
            .ok_or_else(|| anyhow::anyhow!("failed to resolve SSH address"))?;

        let tcp = TcpStream::connect_timeout(&addr, CONNECT_TIMEOUT)
            .map_err(|e| anyhow::anyhow!("failed to connect to SSH: {e}"))?;

        let mut session = Session::new().map_err(|e| anyhow::anyhow!("failed to init SSH: {e}"))?;
        session.set_tcp_stream(tcp);
        session.set_timeout(IO_TIMEOUT_MS);
        session
            .handshake()
            .map_err(|e| anyhow::anyhow!("SSH handshake failed: {e}"))?;

        let mut authenticated = false;

        if !self.config.key_file.is_empty() {
            match session.userauth_pubkey_file(
                &self.config.username,
                None,
                Path::new(&self.config.key_file),
                None,
            ) {
                Ok(()) => authenticated = session.authenticated(),
                Err(err) => {
                    if self.config.password.is_empty() {
                        bail!("failed to authenticate with key file: {err}");
                    }
                }
            }
        }

        if !authenticated && !self.config.password.is_empty() {
            session
                .userauth_password(&self.config.username, &self.config.password)
                .map_err(|e| anyhow::anyhow!("SSH password authentication failed: {e}"))?;
            authenticated = session.authenticated();
        }

        if !authenticated {
            bail!("SSH authentication failed");
        }

        let sftp = session
            .sftp()
            .map_err(|e| anyhow::anyhow!("failed to create SFTP client: {e}"))?;

        self.session = Some(session);
        self.sftp = Some(sftp);
        Ok(())
    }

    fn list(&mut self) -> Result<Vec<FileInfo>> {
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
        let sftp = match self.sftp.as_ref() {
            Some(sftp) => sftp,
            None => bail!("not connected"),
        };
        let mut file = sftp
            .open(Path::new(&full_path))
            .map_err(|e| anyhow::anyhow!("failed to open {remote_path}: {e}"))?;
        io::copy(&mut file, w)?;
        Ok(())
    }

    fn upload(&mut self, r: &mut dyn Read, remote_path: &str) -> Result<()> {
        let full_path = join_posix(&self.config.remote_path, remote_path);
        if let Some((dir, _)) = full_path.rsplit_once('/') {
            self.ensure_remote_dir(dir)?;
        }

        let sftp = match self.sftp.as_ref() {
            Some(sftp) => sftp,
            None => bail!("not connected"),
        };
        let mut file = sftp
            .create(Path::new(&full_path))
            .map_err(|e| anyhow::anyhow!("failed to create {remote_path}: {e}"))?;
        io::copy(r, &mut file)?;
        Ok(())
    }

    fn close(&mut self) -> Result<()> {
        if let Some(mut sftp) = self.sftp.take() {
            let _ = sftp.shutdown();
        }
        self.session = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_includes_host_and_default_port() {
        let cfg = Config {
            connector_type: "sftp".to_string(),
            host: "localhost".to_string(),
            ..Default::default()
        };
        let conn = SftpConnector::new(cfg);
        assert_eq!(conn.name(), "sftp://localhost:22");
    }

    #[test]
    fn key_file_config_is_accepted() {
        let cfg = Config {
            connector_type: "sftp".to_string(),
            host: "localhost".to_string(),
            key_file: "/home/user/.ssh/id_rsa".to_string(),
            remote_path: "/saves/".to_string(),
            ..Default::default()
        };
        let conn = SftpConnector::new(cfg);
        assert_eq!(conn.name(), "sftp://localhost:22");
    }

    #[test]
    fn connect_without_auth_errors() {
        let cfg = Config {
            connector_type: "sftp".to_string(),
            host: "localhost".to_string(),
            ..Default::default()
        };
        let mut conn = SftpConnector::new(cfg);
        assert!(conn.connect().is_err());
    }
}
