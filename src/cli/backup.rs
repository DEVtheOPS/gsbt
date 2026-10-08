//! `backup` command implementation.

use std::time::Instant;

use anyhow::{bail, Result};

use crate::backup::Manager;
use crate::config::{Defaults, Server};
use crate::connector::Config as ConnectorConfig;
use crate::log::Logger;
use crate::progress;

use super::ConnectorFactory;

/// Options for the `backup` command.
pub struct BackupOptions {
    pub config: String,
    pub output: String,
    pub server: String,
    pub sequential: bool,
}

/// Runs the backup command using the supplied connector factory.
pub fn run_backup(opts: &BackupOptions, logger: &Logger, factory: &ConnectorFactory) -> Result<()> {
    let cfg_path = crate::config::find_config_file(&opts.config)?;
    let cfg = crate::config::load_config(&cfg_path)?;

    let mut servers = cfg.servers.clone();
    if !opts.server.is_empty() {
        servers.retain(|server| server.name == opts.server);
        if servers.is_empty() {
            bail!("server {:?} not found in config", opts.server);
        }
    }
    if servers.is_empty() {
        bail!("no servers configured");
    }

    let format = opts.output.as_str();

    let run_one = |server: &Server| -> bool {
        let server_logger =
            logger.with_prefix(format!("[bold][cyan]{}[/cyan][/bold]", server.name));
        server_logger.info("[yellow]starting backup[/yellow]");

        let conn_cfg = match to_connector_config(server, &cfg.defaults) {
            Ok(cfg) => cfg,
            Err(err) => {
                server_logger.error(format!("[red]config error:[/red] {err}"));
                return false;
            }
        };

        let mut conn = match factory(conn_cfg) {
            Ok(conn) => conn,
            Err(err) => {
                server_logger.error(format!("[red]init error:[/red] {err}"));
                return false;
            }
        };

        let mut manager = Manager {
            backup_location: server.get_backup_location(&cfg.defaults),
            temp_dir: cfg.defaults.temp_dir.clone(),
            progress: progress::new(&server_logger, format),
        };

        let start = Instant::now();
        match manager.backup(&mut *conn) {
            Ok((archive_path, stats)) => {
                server_logger.info_with(
                    format!(
                        "[green]saved[/green] {archive_path} ({} files, {:.1} MB, {:.1}s)",
                        stats.files,
                        stats.bytes as f64 / 1e6,
                        start.elapsed().as_secs_f64()
                    ),
                    crate::meta! {
                        "archive_path" => archive_path,
                        "files" => stats.files,
                        "bytes" => stats.bytes,
                        "duration_sec" => start.elapsed().as_secs_f64(),
                    },
                );
                true
            }
            Err(err) => {
                server_logger.error(format!("[red]backup failed:[/red] {err}"));
                false
            }
        }
    };

    let (successes, failures) = if opts.sequential || servers.len() == 1 {
        let mut ok = 0usize;
        let mut failed = 0usize;
        for server in &servers {
            if run_one(server) {
                ok += 1;
            } else {
                failed += 1;
            }
        }
        (ok, failed)
    } else {
        let results = std::thread::scope(|scope| {
            let mut handles = Vec::with_capacity(servers.len());
            for server in &servers {
                handles.push(scope.spawn(move || run_one(server)));
            }
            handles
                .into_iter()
                .map(|handle| handle.join().unwrap_or(false))
                .collect::<Vec<bool>>()
        });

        (
            results.iter().filter(|ok| **ok).count(),
            results.iter().filter(|ok| !**ok).count(),
        )
    };

    if failures > 0 {
        bail!("backup complete with failures: {successes} success, {failures} failed");
    }

    logger.info(format!(
        "[bold][green]backup complete[/green][/bold] ({successes} success)"
    ));

    Ok(())
}

/// Converts a server configuration into a connector configuration.
pub fn to_connector_config(server: &Server, defaults: &Defaults) -> Result<ConnectorConfig> {
    let conn = &server.connection;

    let include = conn.get_include();
    let exclude = conn.exclude.clone();

    let api_key = if conn.api_key.is_empty() {
        defaults.nitrado_api_key.clone()
    } else {
        conn.api_key.clone()
    };

    let cfg = ConnectorConfig {
        connector_type: conn.connection_type.clone(),
        host: conn.host.clone(),
        port: conn.port,
        username: conn.username.clone(),
        password: conn.password.clone(),
        key_file: conn.key_file.clone(),
        passive: conn.is_passive(),
        tls: conn.tls,
        api_key,
        service_id: conn.service_id.clone(),
        remote_path: conn.remote_path.clone(),
        include,
        exclude,
        retry_attempts: defaults.retry_attempts,
        retry_delay: defaults.retry_delay,
        retry_backoff: defaults.retry_backoff,
    };

    if cfg.remote_path.is_empty() {
        bail!("connection.remote_path is required");
    }

    Ok(cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requires_remote_path() {
        let server = Server {
            connection: crate::config::Connection {
                connection_type: "ftp".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(to_connector_config(&server, &Defaults::default()).is_err());
    }
}
