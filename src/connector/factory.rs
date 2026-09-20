//! Connector factory.

use anyhow::{bail, Result};

use super::{ftp::FtpConnector, nitrado::NitradoConnector, sftp::SftpConnector, Config, Connector};

/// Instantiates the correct connector implementation based on `cfg.connector_type`.
pub fn new_connector(cfg: Config) -> Result<Box<dyn Connector>> {
    match cfg.connector_type.as_str() {
        "ftp" => Ok(Box::new(FtpConnector::new(cfg))),
        "sftp" => Ok(Box::new(SftpConnector::new(cfg))),
        "nitrado" => Ok(Box::new(NitradoConnector::new(cfg))),
        other => bail!("unsupported connector type: {other}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_expected_connector() {
        for (kind, expected) in [
            ("ftp", "ftp://localhost:21"),
            ("sftp", "sftp://localhost:22"),
            ("nitrado", "nitrado://12345"),
        ] {
            let cfg = Config {
                connector_type: kind.to_string(),
                host: "localhost".to_string(),
                service_id: "12345".to_string(),
                ..Default::default()
            };
            let conn = new_connector(cfg).expect("connector");
            assert_eq!(conn.name(), expected);
        }
    }

    #[test]
    fn unknown_type_errors() {
        let cfg = Config {
            connector_type: "unknown".to_string(),
            ..Default::default()
        };
        assert!(new_connector(cfg).is_err());
    }
}
