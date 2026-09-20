//! Environment variable substitution for configuration values.

use std::env;
use std::sync::LazyLock;

use regex::{Captures, Regex};

use super::types::Config;

/// Matches `${VAR}` and `${VAR:-default}` patterns.
static ENV_VAR_BRACE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\$\{([A-Z_][A-Z0-9_]*)(:-([^}]*))?\}").expect("valid env brace regex")
});

/// Matches `$VAR` patterns.
static ENV_VAR_SIMPLE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\$([A-Z_][A-Z0-9_]*)").expect("valid env simple regex"));

/// Expands environment variables in a string.
///
/// Supports three patterns:
/// - `${VAR}` - replaced with the value of `VAR`, empty if unset.
/// - `${VAR:-default}` - replaced with the value of `VAR`, or `default`.
/// - `$VAR` - replaced with the value of `VAR`, empty if unset.
pub fn expand_env_vars(s: &str) -> String {
    let result = ENV_VAR_BRACE_REGEX.replace_all(s, |caps: &Captures<'_>| {
        let name = &caps[1];
        let default = caps.get(3).map(|m| m.as_str()).unwrap_or("");
        match env::var(name) {
            Ok(value) => value,
            Err(_) => default.to_string(),
        }
    });

    ENV_VAR_SIMPLE_REGEX
        .replace_all(&result, |caps: &Captures<'_>| {
            let name = &caps[1];
            env::var(name).unwrap_or_default()
        })
        .into_owned()
}

/// Expands environment variables across every field of a [`Config`].
pub fn expand_env_vars_in_config(cfg: &mut Config) {
    cfg.defaults.backup_location = expand_env_vars(&cfg.defaults.backup_location);
    cfg.defaults.temp_dir = expand_env_vars(&cfg.defaults.temp_dir);
    cfg.defaults.env_file = expand_env_vars(&cfg.defaults.env_file);
    cfg.defaults.nitrado_api_key = expand_env_vars(&cfg.defaults.nitrado_api_key);

    for server in &mut cfg.servers {
        server.name = expand_env_vars(&server.name);
        server.description = expand_env_vars(&server.description);
        server.backup_location = expand_env_vars(&server.backup_location);

        let conn = &mut server.connection;
        conn.connection_type = expand_env_vars(&conn.connection_type);
        conn.host = expand_env_vars(&conn.host);
        conn.username = expand_env_vars(&conn.username);
        conn.password = expand_env_vars(&conn.password);
        conn.key_file = expand_env_vars(&conn.key_file);
        conn.api_key = expand_env_vars(&conn.api_key);
        conn.service_id = expand_env_vars(&conn.service_id);
        conn.remote_path = expand_env_vars(&conn.remote_path);

        for pattern in &mut conn.include {
            *pattern = expand_env_vars(pattern);
        }
        for pattern in &mut conn.exclude {
            *pattern = expand_env_vars(pattern);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::{Connection, Defaults, Server};

    #[test]
    #[allow(clippy::type_complexity)]
    fn expand_cases() {
        let cases: &[(&str, &[(&str, &str)], &str)] = &[
            (
                "password: ${GSBT_TEST_PASSWORD}",
                &[("GSBT_TEST_PASSWORD", "secret123")],
                "password: secret123",
            ),
            (
                "user: $GSBT_TEST_USERNAME",
                &[("GSBT_TEST_USERNAME", "admin")],
                "user: admin",
            ),
            (
                "key: ${GSBT_TEST_MISSING_KEY:-default_key}",
                &[],
                "key: default_key",
            ),
            (
                "multiple: ${GSBT_TEST_HOST}, user: ${GSBT_TEST_USER}",
                &[("GSBT_TEST_HOST", "localhost"), ("GSBT_TEST_USER", "admin")],
                "multiple: localhost, user: admin",
            ),
            (
                "connect: $GSBT_TEST_USER@${GSBT_TEST_HOST}:${GSBT_TEST_PORT:-22}",
                &[
                    ("GSBT_TEST_USER", "admin"),
                    ("GSBT_TEST_HOST", "192.168.1.1"),
                ],
                "connect: admin@192.168.1.1:22",
            ),
            ("plain: text", &[], "plain: text"),
            ("value: ${GSBT_TEST_MISSING}", &[], "value: "),
            ("price: $5.00", &[], "price: $5.00"),
        ];

        for (input, vars, expected) in cases {
            for (k, v) in *vars {
                env::set_var(k, v);
            }
            assert_eq!(&expand_env_vars(input), expected, "input: {input}");
            for (k, _) in *vars {
                env::remove_var(k);
            }
        }
    }

    #[test]
    fn default_ignored_when_var_exists() {
        env::set_var("GSBT_TEST_REAL_KEY", "real_key");
        assert_eq!(
            expand_env_vars("key: ${GSBT_TEST_REAL_KEY:-default_key}"),
            "key: real_key"
        );
        env::remove_var("GSBT_TEST_REAL_KEY");
    }

    #[test]
    fn expand_in_config() {
        env::set_var("GSBT_TEST_BACKUP_LOCATION", "/srv/backups");
        env::set_var("GSBT_TEST_FTP_PASSWORD", "ftp_pass123");
        env::set_var("GSBT_TEST_NITRADO_KEY", "nitrado_key_456");

        let mut cfg = Config {
            defaults: Defaults {
                backup_location: "${GSBT_TEST_BACKUP_LOCATION}".into(),
                temp_dir: "${GSBT_TEST_TEMP_DIR:-/tmp}".into(),
                prune_age: 30,
                nitrado_api_key: "${GSBT_TEST_NITRADO_KEY}".into(),
                ..Default::default()
            },
            servers: vec![Server {
                name: "test-server".into(),
                backup_location: "${GSBT_TEST_BACKUP_LOCATION}/test".into(),
                connection: Connection {
                    connection_type: "ftp".into(),
                    host: "localhost".into(),
                    username: "user".into(),
                    password: "${GSBT_TEST_FTP_PASSWORD}".into(),
                    ..Default::default()
                },
                ..Default::default()
            }],
        };

        expand_env_vars_in_config(&mut cfg);

        assert_eq!(cfg.defaults.backup_location, "/srv/backups");
        assert_eq!(cfg.defaults.temp_dir, "/tmp");
        assert_eq!(cfg.defaults.nitrado_api_key, "nitrado_key_456");
        assert_eq!(cfg.servers[0].backup_location, "/srv/backups/test");
        assert_eq!(cfg.servers[0].connection.password, "ftp_pass123");

        env::remove_var("GSBT_TEST_BACKUP_LOCATION");
        env::remove_var("GSBT_TEST_FTP_PASSWORD");
        env::remove_var("GSBT_TEST_NITRADO_KEY");
    }
}
