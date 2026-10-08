//! Configuration data model.

use std::path::Path;

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};

/// The root configuration structure.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct Config {
    /// Default values applied to all servers.
    pub defaults: Defaults,
    /// Configured game servers.
    pub servers: Vec<Server>,
}

/// Default values shared by every server.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct Defaults {
    /// Base directory where backups are stored.
    pub backup_location: String,
    /// Directory used for temporary download files.
    pub temp_dir: String,
    /// Number of days backups are retained.
    pub prune_age: i32,
    /// Number of retry attempts for transient failures.
    pub retry_attempts: i32,
    /// Delay in seconds between retries.
    pub retry_delay: i32,
    /// Whether retries use exponential backoff.
    pub retry_backoff: bool,
    /// Optional `.env` file to load before expanding variables.
    pub env_file: String,
    /// Default Nitrado API key.
    pub nitrado_api_key: String,
}

/// A single game server configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct Server {
    /// Human readable server name (used as backup sub-directory).
    pub name: String,
    /// Optional description.
    pub description: String,
    /// Server-specific backup location override.
    pub backup_location: String,
    /// Server-specific prune age override.
    pub prune_age: i32,
    /// Connector configuration.
    pub connection: Connection,
}

/// Connector-specific configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct Connection {
    /// Connector type: `ftp`, `sftp` or `nitrado`.
    #[serde(rename = "type")]
    pub connection_type: String,
    /// Remote host.
    pub host: String,
    /// Remote port.
    pub port: u16,
    /// Username for authentication.
    pub username: String,
    /// Password for authentication.
    pub password: String,
    /// Private key file (SFTP).
    pub key_file: String,
    /// FTP passive mode (defaults to `true`).
    pub passive: Option<bool>,
    /// Use explicit TLS for FTP.
    pub tls: bool,
    /// API key (Nitrado).
    pub api_key: String,
    /// Service identifier (Nitrado). Accepts a string or integer in YAML.
    #[serde(deserialize_with = "string_or_int")]
    pub service_id: String,
    /// Path on the remote server that is backed up.
    pub remote_path: String,
    /// Glob patterns to include.
    pub include: Vec<String>,
    /// Glob patterns to exclude.
    pub exclude: Vec<String>,
}

impl Server {
    /// Returns the server-specific backup location, or the default with the
    /// server name appended.
    pub fn get_backup_location(&self, defaults: &Defaults) -> String {
        if !self.backup_location.is_empty() {
            return self.backup_location.clone();
        }
        if defaults.backup_location.is_empty() {
            return String::new();
        }
        if self.name.is_empty() {
            return defaults.backup_location.clone();
        }
        Path::new(&defaults.backup_location)
            .join(&self.name)
            .to_string_lossy()
            .into_owned()
    }

    /// Returns the server-specific prune age, or the default.
    pub fn get_prune_age(&self, defaults: &Defaults) -> i32 {
        if self.prune_age > 0 {
            self.prune_age
        } else {
            defaults.prune_age
        }
    }
}

impl Connection {
    /// Returns the include patterns, defaulting to `["*"]`.
    pub fn get_include(&self) -> Vec<String> {
        if !self.include.is_empty() {
            self.include.clone()
        } else {
            vec!["*".to_string()]
        }
    }

    /// Returns the passive mode setting (defaults to `true`).
    pub fn is_passive(&self) -> bool {
        self.passive.unwrap_or(true)
    }
}

/// Deserialises a value that may be either a YAML string or integer into a
/// `String`. This mirrors the permissive handling of `service_id` values.
fn string_or_int<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt {
        String(String),
        Integer(i64),
    }

    Ok(match StringOrInt::deserialize(deserializer)? {
        StringOrInt::String(s) => s,
        StringOrInt::Integer(i) => i.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_parsing() {
        let yaml_data = r#"
defaults:
  backup_location: /srv/backups/
  prune_age: 30
  retry_attempts: 3

servers:
  - name: test-server
    description: Test Server
    backup_location: /custom/location
    connection:
      type: ftp
      host: localhost
      username: user
      password: pass
      remote_path: /saves/
"#;
        let cfg: Config = serde_yaml::from_str(yaml_data).expect("parse yaml");

        assert_eq!(cfg.defaults.backup_location, "/srv/backups/");
        assert_eq!(cfg.servers.len(), 1);
        assert_eq!(cfg.servers[0].name, "test-server");
        assert_eq!(
            cfg.servers[0].get_backup_location(&cfg.defaults),
            "/custom/location"
        );
        assert_eq!(cfg.servers[0].connection.connection_type, "ftp");
    }

    #[test]
    fn service_id_accepts_integer() {
        let yaml_data = r#"
servers:
  - name: nitrado
    connection:
      type: nitrado
      service_id: 18341077
"#;
        let cfg: Config = serde_yaml::from_str(yaml_data).expect("parse yaml");
        assert_eq!(cfg.servers[0].connection.service_id, "18341077");
    }

    #[test]
    fn derived_backup_location() {
        let defaults = Defaults {
            backup_location: "/srv/backups/".into(),
            ..Default::default()
        };
        let server = Server {
            name: "test".into(),
            ..Default::default()
        };
        assert_eq!(
            server.get_backup_location(&defaults),
            Path::new("/srv/backups/")
                .join("test")
                .to_string_lossy()
                .into_owned()
        );
    }

    #[test]
    fn prune_age_and_include_defaults() {
        let defaults = Defaults {
            prune_age: 30,
            ..Default::default()
        };
        let server = Server::default();
        assert_eq!(server.get_prune_age(&defaults), 30);

        let connection = Connection::default();
        assert_eq!(connection.get_include(), vec!["*".to_string()]);
        assert!(connection.is_passive());
    }
}
