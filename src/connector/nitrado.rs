//! Nitrado connector.
//!
//! Fetches FTP credentials from the Nitrado API and delegates the actual
//! transfer to the FTP connector.

use std::io::{Read, Write};

use anyhow::{bail, Result};
use serde::Deserialize;

use super::ftp::FtpConnector;
use super::{Config, Connector, FileInfo};

const NITRADO_API_BASE: &str = "https://api.nitrado.net";

#[derive(Debug, Default, Deserialize)]
struct NitradoFtpResponse {
    #[serde(default)]
    status: String,
    #[serde(default)]
    data: NitradoData,
    #[serde(default)]
    message: String,
}

#[derive(Debug, Default, Deserialize)]
struct NitradoData {
    #[serde(default)]
    ftp: NitradoFtp,
}

#[derive(Debug, Default, Deserialize)]
struct NitradoFtp {
    #[serde(default)]
    hostname: String,
    #[serde(default)]
    port: u16,
    #[serde(default)]
    username: String,
    #[serde(default)]
    password: String,
}

struct FtpCredentials {
    hostname: String,
    port: u16,
    username: String,
    password: String,
}

/// Nitrado connector implementation.
pub struct NitradoConnector {
    config: Config,
    ftp: Option<FtpConnector>,
    api_key: String,
    service_id: String,
    api_base: String,
}

impl NitradoConnector {
    /// Creates a new Nitrado connector.
    pub fn new(cfg: Config) -> Self {
        Self {
            api_key: cfg.api_key.clone(),
            service_id: cfg.service_id.clone(),
            config: cfg,
            ftp: None,
            api_base: NITRADO_API_BASE.to_string(),
        }
    }

    fn fetch_ftp_credentials(&self) -> Result<FtpCredentials> {
        let url = format!("{}/services/{}/gameservers", self.api_base, self.service_id);

        let response = ureq::get(&url)
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .set("Accept", "application/json")
            .call();

        let response = match response {
            Ok(response) => response,
            Err(ureq::Error::Status(429, response)) => {
                let retry_after = response
                    .header("Retry-After")
                    .unwrap_or("unknown")
                    .to_string();
                bail!("rate limited by Nitrado API (retry after: {retry_after})");
            }
            Err(ureq::Error::Status(code, response)) => {
                let body = response.into_string().unwrap_or_default();
                bail!("Nitrado API error (status {code}): {body}");
            }
            Err(err) => return Err(anyhow::anyhow!(err)),
        };

        let parsed: NitradoFtpResponse = response
            .into_json()
            .map_err(|e| anyhow::anyhow!("failed to parse Nitrado response: {e}"))?;

        if parsed.status != "success" {
            bail!("Nitrado API returned error: {}", parsed.message);
        }

        Ok(FtpCredentials {
            hostname: parsed.data.ftp.hostname,
            port: parsed.data.ftp.port,
            username: parsed.data.ftp.username,
            password: parsed.data.ftp.password,
        })
    }
}

impl Connector for NitradoConnector {
    fn name(&self) -> String {
        format!("nitrado://{}", self.service_id)
    }

    fn connect(&mut self) -> Result<()> {
        if self.api_key.is_empty() {
            bail!("api_key is required for nitrado connector");
        }
        if self.service_id.is_empty() {
            bail!("service_id is required for nitrado connector");
        }

        let creds = self
            .fetch_ftp_credentials()
            .map_err(|e| anyhow::anyhow!("failed to get Nitrado FTP credentials: {e}"))?;

        let ftp_config = Config {
            connector_type: "ftp".to_string(),
            host: creds.hostname,
            port: creds.port,
            username: creds.username,
            password: creds.password,
            remote_path: self.config.remote_path.clone(),
            include: self.config.include.clone(),
            exclude: self.config.exclude.clone(),
            passive: true,
            retry_attempts: self.config.retry_attempts,
            retry_delay: self.config.retry_delay,
            retry_backoff: self.config.retry_backoff,
            ..Default::default()
        };

        let mut ftp = FtpConnector::from_config(ftp_config);
        ftp.connect()?;
        self.ftp = Some(ftp);
        Ok(())
    }

    fn list(&mut self) -> Result<Vec<FileInfo>> {
        match self.ftp.as_mut() {
            Some(ftp) => ftp.list(),
            None => bail!("not connected"),
        }
    }

    fn download(&mut self, remote_path: &str, w: &mut dyn Write) -> Result<()> {
        match self.ftp.as_mut() {
            Some(ftp) => ftp.download(remote_path, w),
            None => bail!("not connected"),
        }
    }

    fn upload(&mut self, r: &mut dyn Read, remote_path: &str) -> Result<()> {
        match self.ftp.as_mut() {
            Some(ftp) => ftp.upload(r, remote_path),
            None => bail!("not connected"),
        }
    }

    fn close(&mut self) -> Result<()> {
        if let Some(ftp) = self.ftp.as_mut() {
            ftp.close()
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_uses_service_id() {
        let cfg = Config {
            connector_type: "nitrado".to_string(),
            api_key: "test-api-key".to_string(),
            service_id: "12345".to_string(),
            remote_path: "/games/ark/".to_string(),
            include: vec!["*".to_string()],
            exclude: vec!["*.log".to_string()],
            ..Default::default()
        };
        let conn = NitradoConnector::new(cfg);
        assert_eq!(conn.name(), "nitrado://12345");
    }

    #[test]
    fn connect_requires_api_key() {
        let cfg = Config {
            connector_type: "nitrado".to_string(),
            service_id: "12345".to_string(),
            ..Default::default()
        };
        let mut conn = NitradoConnector::new(cfg);
        assert!(conn.connect().is_err());
    }

    #[test]
    fn connect_requires_service_id() {
        let cfg = Config {
            connector_type: "nitrado".to_string(),
            api_key: "key".to_string(),
            ..Default::default()
        };
        let mut conn = NitradoConnector::new(cfg);
        assert!(conn.connect().is_err());
    }
}
