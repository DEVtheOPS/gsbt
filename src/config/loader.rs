//! Configuration file discovery and loading.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use super::envsubst::{expand_env_vars, expand_env_vars_in_config};
use super::types::Config;

/// Locates the config file using the discovery order:
///
/// 1. Explicit path (from `--config`).
/// 2. `GSBT_CONFIG` environment variable.
/// 3. `./.gsbt-config.yml` (current directory).
/// 4. `~/.config/gsbt/config.yml` (user config).
pub fn find_config_file(explicit: &str) -> Result<PathBuf> {
    if !explicit.is_empty() {
        let path = PathBuf::from(explicit);
        if !path.exists() {
            bail!("config file not found: {explicit}");
        }
        return Ok(path);
    }

    if let Ok(env_path) = std::env::var("GSBT_CONFIG") {
        if !env_path.is_empty() {
            let path = PathBuf::from(&env_path);
            if path.exists() {
                return Ok(path);
            }
        }
    }

    let local = Path::new(".gsbt-config.yml");
    if local.exists() {
        let cwd = std::env::current_dir().context("failed to resolve current directory")?;
        return Ok(cwd.join(local));
    }

    if let Some(home) = home_dir() {
        let user_path = home.join(".config").join("gsbt").join("config.yml");
        if user_path.exists() {
            return Ok(user_path);
        }
    }

    bail!("no config file found")
}

/// Loads and parses the config file, expanding env vars and applying defaults.
pub fn load_config(path: impl AsRef<Path>) -> Result<Config> {
    let data = fs::read_to_string(path.as_ref())
        .with_context(|| format!("failed to read config: {}", path.as_ref().display()))?;

    let mut cfg: Config = serde_yaml::from_str(&data).context("failed to parse config")?;

    if !cfg.defaults.env_file.is_empty() {
        let env_path = expand_env_vars(&cfg.defaults.env_file);
        // Non-fatal: the env file is optional.
        let _ = dotenvy::from_path(&env_path);
    }

    expand_env_vars_in_config(&mut cfg);
    apply_defaults(&mut cfg);

    Ok(cfg)
}

fn apply_defaults(cfg: &mut Config) {
    if cfg.defaults.retry_attempts == 0 {
        cfg.defaults.retry_attempts = 3;
    }
    if cfg.defaults.retry_delay == 0 {
        cfg.defaults.retry_delay = 5;
    }
    if cfg.defaults.prune_age == 0 {
        cfg.defaults.prune_age = 30;
    }
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_config(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        let mut file = fs::File::create(&path).expect("create config");
        file.write_all(content.as_bytes()).expect("write config");
        path
    }

    #[test]
    fn find_config_file_explicit_and_env() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let test_config = write_config(
            tmp.path(),
            ".gsbt-config.yml",
            "defaults:\n  prune_age: 30\n",
        );

        let found = find_config_file(test_config.to_str().unwrap()).expect("explicit path");
        assert_eq!(found, test_config);

        std::env::set_var("GSBT_CONFIG", &test_config);
        let found = find_config_file("").expect("env path");
        assert_eq!(found, test_config);
        std::env::remove_var("GSBT_CONFIG");
    }

    #[test]
    fn load_config_parses_values() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let content = "
defaults:
  backup_location: /srv/backups/
  prune_age: 30

servers:
  - name: test
    connection:
      type: ftp
      host: localhost
      username: user
      password: pass
      remote_path: /saves/
";
        let path = write_config(tmp.path(), "config.yml", content);
        let cfg = load_config(&path).expect("load config");

        assert_eq!(cfg.defaults.prune_age, 30);
        assert_eq!(cfg.defaults.backup_location, "/srv/backups/");
        assert_eq!(
            cfg.servers[0].get_backup_location(&cfg.defaults),
            Path::new("/srv/backups/")
                .join("test")
                .to_string_lossy()
                .into_owned()
        );
        assert_eq!(cfg.servers.len(), 1);
        assert_eq!(cfg.servers[0].name, "test");
    }

    #[test]
    fn load_config_applies_defaults() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let content = "
defaults:
  backup_location: /srv/backups/

servers:
  - name: test
    connection:
      type: ftp
      host: localhost
";
        let path = write_config(tmp.path(), "config.yml", content);
        let cfg = load_config(&path).expect("load config");

        assert_eq!(cfg.defaults.retry_attempts, 3);
        assert_eq!(cfg.defaults.retry_delay, 5);
        assert_eq!(cfg.defaults.prune_age, 30);
    }

    #[test]
    fn load_config_expands_env_vars() {
        std::env::set_var("GSBT_TEST_BACKUP_PATH", "/test/backups");
        std::env::set_var("GSBT_TEST_FTP_HOST", "test.example.com");

        let tmp = tempfile::tempdir().expect("tempdir");
        let content = "
defaults:
  backup_location: ${GSBT_TEST_BACKUP_PATH}

servers:
  - name: test
    connection:
      type: ftp
      host: ${GSBT_TEST_FTP_HOST}
";
        let path = write_config(tmp.path(), "config.yml", content);
        let cfg = load_config(&path).expect("load config");

        assert_eq!(cfg.defaults.backup_location, "/test/backups");
        assert_eq!(cfg.servers[0].connection.host, "test.example.com");

        std::env::remove_var("GSBT_TEST_BACKUP_PATH");
        std::env::remove_var("GSBT_TEST_FTP_HOST");
    }
}
