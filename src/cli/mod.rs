//! Command line interface.

pub mod backup;

use std::fs;
use std::io::Write;
use std::path::Path;

use anyhow::{bail, Context, Result};
use clap::{CommandFactory, Parser, Subcommand};

use crate::connector::{Config as ConnectorConfig, Connector};
use crate::log::Logger;

use backup::BackupOptions;

/// Factory used to create connectors. Injectable to allow testing.
pub type ConnectorFactory = dyn Fn(ConnectorConfig) -> Result<Box<dyn Connector>> + Send + Sync;

/// Build metadata, overridable at compile time.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const BUILD_DATE: &str = match option_env!("GSBT_BUILD_DATE") {
    Some(value) => value,
    None => "unknown",
};
pub const COMMIT: &str = match option_env!("GSBT_COMMIT") {
    Some(value) => value,
    None => "unknown",
};
pub const BRANCH: &str = match option_env!("GSBT_BRANCH") {
    Some(value) => value,
    None => "unknown",
};

/// Gameserver Backup Tool.
#[derive(Debug, Parser)]
#[command(
    name = "gsbt",
    about = "Gameserver Backup Tool",
    long_about = "A modular backup tool for game servers supporting FTP, SFTP, and Nitrado.",
    disable_version_flag = true
)]
pub struct Cli {
    /// config file path
    #[arg(long, global = true, default_value = "")]
    pub config: String,
    /// output format (text, json)
    #[arg(long, global = true, default_value = "text")]
    pub output: String,
    /// verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
    /// suppress non-error output
    #[arg(short, long, global = true)]
    pub quiet: bool,
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Available subcommands.
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Backup gameserver files
    #[command(
        about = "Backup gameserver files",
        long_about = "Download and archive files from configured gameservers."
    )]
    Backup {
        /// backup specific server only
        #[arg(long, default_value = "")]
        server: String,
        /// run backups sequentially
        #[arg(long)]
        sequential: bool,
    },
    /// Remove old backups
    #[command(
        about = "Remove old backups",
        long_about = "Delete backups older than the configured prune_age."
    )]
    Prune {
        /// prune specific server only
        #[arg(long, default_value = "")]
        server: String,
        /// show what would be deleted
        #[arg(long)]
        dry_run: bool,
    },
    /// List configured servers
    #[command(
        about = "List configured servers",
        long_about = "Show configured servers and their backup status."
    )]
    List {
        /// show specific server details
        #[arg(long, default_value = "")]
        server: String,
    },
    /// Restore a backup
    #[command(
        about = "Restore a backup",
        long_about = "Restore a backup to a server or extract locally."
    )]
    Restore {
        /// backup file to restore
        backup_file: String,
        /// restore to server
        #[arg(long, default_value = "")]
        server: String,
        /// extract to local path
        #[arg(long, default_value = "")]
        local: String,
        /// show what would be restored
        #[arg(long)]
        dry_run: bool,
        /// skip confirmation prompt
        #[arg(long)]
        force: bool,
    },
    /// Initialize a new configuration file
    #[command(
        about = "Initialize a new configuration file",
        long_about = "Creates a new example configuration file with defaults and comments."
    )]
    Init {
        /// output file path
        #[arg(short, long, default_value = ".gsbt-config.yml")]
        outfile: String,
        /// overwrite existing config file
        #[arg(short, long)]
        force: bool,
    },
    /// Print version information
    Version,
}

/// Parses CLI arguments and executes the requested command.
pub fn run() -> Result<()> {
    let cli = Cli::parse();
    execute(
        &cli,
        &crate::connector::new_connector,
        Box::new(std::io::stdout()),
        Box::new(std::io::stderr()),
    )
}

/// Executes a parsed CLI invocation. All output is written to `out`/`err`,
/// which makes the function straightforward to test.
pub fn execute(
    cli: &Cli,
    factory: &ConnectorFactory,
    mut out: Box<dyn Write + Send>,
    err: Box<dyn Write + Send>,
) -> Result<()> {
    if cli.verbose && cli.quiet {
        bail!("--verbose and --quiet flags cannot be used together");
    }

    match cli.command.as_ref() {
        None => {
            let mut command = Cli::command();
            command.write_help(&mut out)?;
            Ok(())
        }
        Some(Commands::Version) => {
            print_version(&cli.output, &mut *out)?;
            Ok(())
        }
        Some(Commands::Init { outfile, force }) => init_config(outfile, *force, &mut *out),
        Some(Commands::List { .. }) => {
            writeln!(out, "list command - not yet implemented")?;
            Ok(())
        }
        Some(Commands::Prune { .. }) => {
            writeln!(out, "prune command - not yet implemented")?;
            Ok(())
        }
        Some(Commands::Restore { backup_file, .. }) => {
            writeln!(
                out,
                "restore command - not yet implemented (file: {backup_file})"
            )?;
            Ok(())
        }
        Some(Commands::Backup { server, sequential }) => {
            let mut logger = Logger::with_writers(out, err);
            logger.set_output_format(&cli.output);
            logger.set_quiet(cli.quiet);
            logger.set_verbose(cli.verbose);

            let opts = BackupOptions {
                config: cli.config.clone(),
                output: cli.output.clone(),
                server: server.clone(),
                sequential: *sequential,
            };
            backup::run_backup(&opts, &logger, factory)
        }
    }
}

fn print_version(output: &str, out: &mut dyn Write) -> Result<()> {
    if output == "json" {
        let data = serde_json::json!({
            "version": VERSION,
            "commit": COMMIT,
            "build_date": BUILD_DATE,
            "branch": BRANCH,
        });
        writeln!(out, "{}", serde_json::to_string_pretty(&data)?)?;
    } else {
        writeln!(out, "gsbt {VERSION} ({BRANCH}: {COMMIT}) {BUILD_DATE}")?;
    }
    Ok(())
}

const INIT_CONFIG: &str = r#"# yaml-language-server: $schema=https://github.com/devtheops/gsbt/releases/latest/download/gsbt.schema.json

defaults:
  backup_location: ./backups
  temp_dir: ./.tmp
  prune_age: 30
  retry_attempts: 3
  retry_delay: 5
  retry_backoff: true
  # nitrado_api_key: ${NITRADO_API_KEY}

servers:
  - name: example-ftp-server
    description: "An example FTP server backup"
    connection:
      type: ftp
      host: ftp.example.com
      port: 21
      username: user
      password: ${FTP_PASSWORD}
      remote_path: /game/saves
      include: ["*"]
      exclude: ["*.log", "Logs/"]

  # - name: example-nitrado-server
  #   connection:
  #     type: nitrado
  #     service_id: "1234567"
  #     remote_path: /games/ark/saves
"#;

fn init_config(outfile: &str, force: bool, out: &mut dyn Write) -> Result<()> {
    if Path::new(outfile).exists() && !force {
        bail!("file {outfile} already exists; use --force to overwrite");
    }

    if let Some(dir) = Path::new(outfile).parent() {
        if !dir.as_os_str().is_empty() {
            fs::create_dir_all(dir).context("failed to create directory")?;
        }
    }

    fs::write(outfile, INIT_CONFIG).context("failed to write config file")?;
    writeln!(out, "Configuration initialized at {outfile}")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connector::{Connector, FileInfo};
    use crate::log::BufferWriter;
    use std::io::Read;
    use std::time::{Duration, Instant, SystemTime};

    struct MockConnector {
        sleep: Duration,
        files: Vec<FileInfo>,
    }

    impl Connector for MockConnector {
        fn connect(&mut self) -> Result<()> {
            Ok(())
        }

        fn list(&mut self) -> Result<Vec<FileInfo>> {
            Ok(self.files.clone())
        }

        fn download(&mut self, _remote_path: &str, w: &mut dyn Write) -> Result<()> {
            std::thread::sleep(self.sleep);
            w.write_all(b"data")?;
            Ok(())
        }

        fn upload(&mut self, _r: &mut dyn Read, _remote_path: &str) -> Result<()> {
            Ok(())
        }

        fn close(&mut self) -> Result<()> {
            Ok(())
        }

        fn name(&self) -> String {
            "mock".to_string()
        }
    }

    fn file() -> FileInfo {
        FileInfo {
            path: "file.txt".to_string(),
            size: 4,
            mod_time: SystemTime::now(),
            is_dir: false,
        }
    }

    fn factory_with(
        sleep: Duration,
    ) -> impl Fn(ConnectorConfig) -> Result<Box<dyn Connector>> + Send + Sync {
        move |_cfg| {
            Ok(Box::new(MockConnector {
                sleep,
                files: vec![file()],
            }) as Box<dyn Connector>)
        }
    }

    fn write_config(dir: &Path, servers: &[(&str, &str)]) -> std::path::PathBuf {
        let backups = dir.join("backups");
        let mut content = format!(
            "defaults:\n  backup_location: {}\nservers:\n",
            backups.display()
        );
        for (name, _) in servers {
            content.push_str(&format!(
                "  - name: {name}\n    connection:\n      type: ftp\n      host: example.com\n      remote_path: /data\n"
            ));
        }
        let path = dir.join("config.yml");
        fs::write(&path, content).expect("write config");
        path
    }

    fn run(args: &[&str], factory: &ConnectorFactory) -> (Result<()>, String) {
        let cli = Cli::try_parse_from(args).expect("parse args");
        let out = BufferWriter::new();
        let err = BufferWriter::new();
        let result = execute(&cli, factory, Box::new(out.clone()), Box::new(err.clone()));
        let mut combined = out.contents();
        combined.push_str(&err.contents());
        (result, combined)
    }

    #[test]
    fn parses_global_flags() {
        let cli = Cli::try_parse_from([
            "gsbt",
            "--config",
            "/etc/gsbt/config.yaml",
            "--output",
            "json",
            "-v",
        ])
        .expect("parse");
        assert_eq!(cli.config, "/etc/gsbt/config.yaml");
        assert_eq!(cli.output, "json");
        assert!(cli.verbose);
        assert!(!cli.quiet);
    }

    #[test]
    fn verbose_and_quiet_conflict() {
        let factory = factory_with(Duration::ZERO);
        let (result, _) = run(&["gsbt", "backup", "-v", "-q"], &factory);
        assert_eq!(
            result.unwrap_err().to_string(),
            "--verbose and --quiet flags cannot be used together"
        );
    }

    #[test]
    fn backup_success_message() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cfg = write_config(tmp.path(), &[("test", "ftp")]);
        let factory = factory_with(Duration::ZERO);

        let (result, output) = run(
            &["gsbt", "backup", "--config", cfg.to_str().unwrap()],
            &factory,
        );
        result.expect("backup ok");
        assert!(output.contains("backup complete"), "output: {output}");
        assert!(tmp.path().join("backups").join("test").exists());
    }

    #[test]
    fn backup_missing_config_errors() {
        let factory = factory_with(Duration::ZERO);
        let (result, _) = run(&["gsbt", "backup", "--config", "/nonexistent"], &factory);
        assert!(result.is_err());
    }

    #[test]
    fn backup_unknown_server_errors() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cfg = write_config(tmp.path(), &[("test", "ftp")]);
        let factory = factory_with(Duration::ZERO);

        let (result, _) = run(
            &[
                "gsbt",
                "backup",
                "--config",
                cfg.to_str().unwrap(),
                "--server",
                "other",
            ],
            &factory,
        );
        assert!(result.is_err());
    }

    #[test]
    fn backups_run_in_parallel_by_default() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cfg = write_config(tmp.path(), &[("s1", "ftp"), ("s2", "ftp")]);
        let duration = Duration::from_millis(150);
        let factory = factory_with(duration);

        let start = Instant::now();
        let (result, _) = run(
            &["gsbt", "backup", "--config", cfg.to_str().unwrap()],
            &factory,
        );
        let elapsed = start.elapsed();

        result.expect("backup ok");
        assert!(
            elapsed < 2 * duration,
            "expected parallel execution, took {elapsed:?}"
        );
    }

    #[test]
    fn sequential_flag_serialises_backups() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cfg = write_config(tmp.path(), &[("s1", "ftp"), ("s2", "ftp")]);
        let duration = Duration::from_millis(100);
        let factory = factory_with(duration);

        let start = Instant::now();
        let (result, _) = run(
            &[
                "gsbt",
                "backup",
                "--config",
                cfg.to_str().unwrap(),
                "--sequential",
            ],
            &factory,
        );
        let elapsed = start.elapsed();

        result.expect("backup ok");
        assert!(
            elapsed >= 2 * duration,
            "expected sequential execution, took {elapsed:?}"
        );
    }

    #[test]
    fn stub_commands_output() {
        let factory = factory_with(Duration::ZERO);

        let (result, output) = run(&["gsbt", "prune"], &factory);
        result.expect("prune ok");
        assert!(output.contains("prune command - not yet implemented"));

        let (result, output) = run(&["gsbt", "list"], &factory);
        result.expect("list ok");
        assert!(output.contains("list command - not yet implemented"));

        let (result, output) = run(&["gsbt", "restore", "backup.tar.gz"], &factory);
        result.expect("restore ok");
        assert!(output.contains("restore command - not yet implemented (file: backup.tar.gz)"));
    }

    #[test]
    fn restore_requires_exactly_one_argument() {
        assert!(Cli::try_parse_from(["gsbt", "restore"]).is_err());
        assert!(Cli::try_parse_from(["gsbt", "restore", "backup.tar.gz"]).is_ok());
        assert!(Cli::try_parse_from(["gsbt", "restore", "a.tar.gz", "b.tar.gz"]).is_err());
    }

    #[test]
    fn all_commands_are_registered() {
        let command = Cli::command();
        let names: Vec<String> = command
            .get_subcommands()
            .map(|sub| sub.get_name().to_string())
            .collect();

        for expected in ["version", "backup", "prune", "list", "restore"] {
            assert!(
                names.iter().any(|name| name == expected),
                "missing command {expected} in {names:?}"
            );
        }
    }

    #[test]
    fn root_help_lists_commands_and_flags() {
        let factory = factory_with(Duration::ZERO);
        let (result, output) = run(&["gsbt"], &factory);
        result.expect("help ok");

        for expected in [
            "backup",
            "prune",
            "list",
            "restore",
            "version",
            "init",
            "--config",
            "--output",
            "--verbose",
            "--quiet",
        ] {
            assert!(
                output.contains(expected),
                "help missing {expected}\n{output}"
            );
        }
    }

    #[test]
    fn version_output() {
        let factory = factory_with(Duration::ZERO);
        let (result, output) = run(&["gsbt", "version"], &factory);
        result.expect("version ok");
        assert!(output.contains("gsbt"));
        assert!(output.contains(VERSION));

        let (result, output) = run(&["gsbt", "--output", "json", "version"], &factory);
        result.expect("version json ok");
        let parsed: serde_json::Value = serde_json::from_str(output.trim()).expect("json");
        assert_eq!(parsed["version"], VERSION);
    }

    #[test]
    fn init_creates_config() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("out.yml");
        let factory = factory_with(Duration::ZERO);

        let (result, output) = run(
            &["gsbt", "init", "--outfile", path.to_str().unwrap()],
            &factory,
        );
        result.expect("init ok");
        assert!(output.contains("Configuration initialized"));
        assert!(path.exists());

        // Second run without --force fails.
        let (result, _) = run(
            &["gsbt", "init", "--outfile", path.to_str().unwrap()],
            &factory,
        );
        assert!(result.is_err());
    }
}
