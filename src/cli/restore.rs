//! `restore` command implementation.

use std::io::{BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use crate::config::{find_config_file, load_config, Server};
use crate::log::Logger;
use crate::restore::{
    build_plan, execute_local, execute_remote, scan_archive, ConflictSet, ExecutionSummary, Plan,
    RestoreOptions,
};

use super::backup::to_connector_config;
use super::ConnectorFactory;

/// Arguments for the `restore` command.
pub struct RestoreCliArgs {
    pub config: String,
    pub output: String,
    pub backup_file: String,
    pub server: String,
    pub local: String,
    pub dry_run: bool,
    pub overwrite: bool,
    pub force: bool,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub strip_components: usize,
}

/// Runs the restore command.
///
/// Exactly one target mode (`--server` or `--local`) is required. The archive
/// is validated and planned before any write is attempted. Remote restores
/// require interactive confirmation unless `--force` is supplied.
pub fn run_restore(
    args: &RestoreCliArgs,
    logger: &Logger,
    factory: &ConnectorFactory,
) -> Result<()> {
    let has_server = !args.server.is_empty();
    let has_local = !args.local.is_empty();
    if has_server == has_local {
        bail!("exactly one of --server <name> or --local <dir> is required");
    }

    let archive_path = Path::new(&args.backup_file);
    crate::restore::validate_archive(archive_path)?;

    let options = RestoreOptions {
        overwrite: args.overwrite,
        includes: args.include.clone(),
        excludes: args.exclude.clone(),
        strip_components: args.strip_components,
    };

    if has_local {
        restore_local(args, logger, &options, archive_path)
    } else {
        restore_remote(args, logger, factory, &options, archive_path)
    }
}

fn restore_local(
    args: &RestoreCliArgs,
    logger: &Logger,
    options: &RestoreOptions,
    archive_path: &Path,
) -> Result<()> {
    let root = PathBuf::from(&args.local);
    let entries = scan_archive(archive_path)?;
    let plan = build_plan(&entries, options, &|dest| root.join(dest).exists());

    let summary = execute_local(archive_path, &root, &plan, args.dry_run, logger)?;
    finish(args, logger, &plan, summary, "local", &args.local)
}

fn restore_remote(
    args: &RestoreCliArgs,
    logger: &Logger,
    factory: &ConnectorFactory,
    options: &RestoreOptions,
    archive_path: &Path,
) -> Result<()> {
    let cfg_path = find_config_file(&args.config)?;
    let cfg = load_config(&cfg_path)?;

    let matches: Vec<&Server> = cfg
        .servers
        .iter()
        .filter(|server| server.name == args.server)
        .collect();
    if matches.is_empty() {
        bail!("server {:?} not found in config", args.server);
    }
    if matches.len() > 1 {
        bail!("server {:?} is not unique in config", args.server);
    }
    let server = matches[0];

    let conn_cfg = to_connector_config(server, &cfg.defaults)?;
    let mut conn = factory(conn_cfg)?;
    conn.connect().context("failed to connect to server")?;

    let entries = scan_archive(archive_path)?;
    let conflict = match ConflictSet::from_remote(&mut *conn) {
        Ok(conflict) => conflict,
        Err(err) => {
            let _ = conn.close();
            return Err(err);
        }
    };
    let plan = build_plan(&entries, options, &|dest| conflict.contains(dest));

    if args.dry_run {
        let summary = execute_remote(&mut *conn, archive_path, &plan, true, logger)?;
        let _ = conn.close();
        return finish(args, logger, &plan, summary, "server", &args.server);
    }

    if !args.force {
        if let Err(err) = confirm_remote(args, server, &plan) {
            let _ = conn.close();
            return Err(err);
        }
    }

    let summary = execute_remote(&mut *conn, archive_path, &plan, false, logger)?;
    let _ = conn.close();
    finish(args, logger, &plan, summary, "server", &args.server)
}

/// Emits the final summary and maps failures to a non-zero exit status.
fn finish(
    args: &RestoreCliArgs,
    logger: &Logger,
    plan: &Plan,
    summary: ExecutionSummary,
    mode: &str,
    target: &str,
) -> Result<()> {
    let metadata = crate::meta! {
        "archive_path" => args.backup_file.clone(),
        "mode" => mode,
        "target" => target,
        "dry_run" => args.dry_run,
        "overwrite" => args.overwrite,
        "restored" => summary.restored,
        "skipped" => summary.skipped,
        "failed" => summary.failed,
        "unsupported" => summary.unsupported,
        "duplicates" => plan.duplicates.len(),
    };

    if args.dry_run {
        logger.info_with(
            format!(
                "[bold][cyan]restore plan[/cyan][/bold] ({} to restore, {} skipped, {} failed)",
                summary.restored, summary.skipped, summary.failed
            ),
            metadata,
        );
        return Ok(());
    }

    logger.info_with(
        format!(
            "[bold][green]restore complete[/green][/bold] ({} restored, {} skipped, {} failed)",
            summary.restored, summary.skipped, summary.failed
        ),
        metadata,
    );

    if summary.has_failures() {
        bail!(
            "restore completed with failures: {} restored, {} skipped, {} failed",
            summary.restored,
            summary.skipped,
            summary.failed
        );
    }

    Ok(())
}

fn confirm_remote(args: &RestoreCliArgs, server: &Server, plan: &Plan) -> Result<()> {
    if !std::io::stdin().is_terminal() {
        bail!(
            "refusing to restore to server {:?} without --force in a non-interactive environment",
            server.name
        );
    }

    let summary = plan.summary();
    eprintln!(
        "About to restore {} file(s) to server '{}' (remote root: '{}'), overwrite: {}, archive: {}",
        summary.restore,
        server.name,
        server.connection.remote_path,
        args.overwrite,
        args.backup_file
    );
    eprint!("Proceed? [y/N] ");
    std::io::stderr().flush().ok();

    let mut answer = String::new();
    std::io::stdin()
        .lock()
        .read_line(&mut answer)
        .context("failed to read confirmation")?;

    match answer.trim().to_ascii_lowercase().as_str() {
        "y" | "yes" => Ok(()),
        _ => bail!("restore cancelled"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connector::{Config as ConnectorConfig, Connector, FileInfo};
    use crate::log::BufferWriter;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::fs;
    use std::io::Read;
    use std::sync::{Arc, Mutex};
    use std::time::SystemTime;
    use tar::{Builder, Header};

    struct MockConnector {
        files: Vec<FileInfo>,
        uploads: Arc<Mutex<Vec<String>>>,
    }

    impl Connector for MockConnector {
        fn connect(&mut self) -> Result<()> {
            Ok(())
        }
        fn list(&mut self) -> Result<Vec<FileInfo>> {
            Ok(self.files.clone())
        }
        fn download(&mut self, _remote_path: &str, _w: &mut dyn Write) -> Result<()> {
            Ok(())
        }
        fn upload(&mut self, r: &mut dyn Read, remote_path: &str) -> Result<()> {
            let mut data = Vec::new();
            r.read_to_end(&mut data)?;
            self.uploads.lock().unwrap().push(remote_path.to_string());
            Ok(())
        }
        fn close(&mut self) -> Result<()> {
            Ok(())
        }
        fn name(&self) -> String {
            "mock".to_string()
        }
    }

    fn archive(dir: &Path) -> PathBuf {
        let dest = dir.join("backup.tar.gz");
        let file = fs::File::create(&dest).unwrap();
        let encoder = GzEncoder::new(file, Compression::default());
        let mut builder = Builder::new(encoder);

        for (name, data) in [("root.txt", "root"), ("nested/child.txt", "child")] {
            let mut header = Header::new_gnu();
            header.set_size(data.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder
                .append_data(&mut header, name, data.as_bytes())
                .unwrap();
        }

        builder.finish().unwrap();
        let encoder = builder.into_inner().unwrap();
        encoder.finish().unwrap();
        dest
    }

    fn config_file(dir: &Path, remote_path: &str) -> PathBuf {
        let path = dir.join("config.yml");
        fs::write(
            &path,
            format!(
                "defaults:\n  backup_location: {}\nservers:\n  - name: test\n    connection:\n      type: ftp\n      host: example.com\n      remote_path: {}\n",
                dir.join("backups").display(),
                remote_path
            ),
        )
        .unwrap();
        path
    }

    fn args(archive: &Path) -> RestoreCliArgs {
        RestoreCliArgs {
            config: String::new(),
            output: "text".to_string(),
            backup_file: archive.to_string_lossy().into_owned(),
            server: String::new(),
            local: String::new(),
            dry_run: false,
            overwrite: false,
            force: false,
            include: vec![],
            exclude: vec![],
            strip_components: 0,
        }
    }

    fn factory(
        files: Vec<FileInfo>,
        uploads: Arc<Mutex<Vec<String>>>,
    ) -> impl Fn(ConnectorConfig) -> Result<Box<dyn Connector>> + Send + Sync {
        move |_cfg| {
            Ok(Box::new(MockConnector {
                files: files.clone(),
                uploads: uploads.clone(),
            }) as Box<dyn Connector>)
        }
    }

    fn make_logger() -> (Logger, BufferWriter) {
        let out = BufferWriter::new();
        let logger = Logger::with_writers(Box::new(out.clone()), Box::new(out.clone()));
        (logger, out)
    }

    #[test]
    fn requires_exactly_one_mode() {
        let tmp = tempfile::tempdir().unwrap();
        let ar = archive(tmp.path());
        let (logger, _) = make_logger();
        let empty = factory(vec![], Arc::new(Mutex::new(vec![])));

        let none = args(&ar);
        assert!(run_restore(&none, &logger, &empty).is_err());

        let mut both = args(&ar);
        both.server = "test".to_string();
        both.local = tmp.path().to_string_lossy().into_owned();
        assert!(run_restore(&both, &logger, &empty).is_err());
    }

    #[test]
    fn local_dry_run_writes_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let ar = archive(tmp.path());
        let root = tmp.path().join("out");

        let mut a = args(&ar);
        a.local = root.to_string_lossy().into_owned();
        a.dry_run = true;

        let (logger, out) = make_logger();
        let empty = factory(vec![], Arc::new(Mutex::new(vec![])));
        run_restore(&a, &logger, &empty).unwrap();

        assert!(!root.exists());
        assert!(out.contents().contains("restore plan"));
    }

    #[test]
    fn local_restore_extracts_files() {
        let tmp = tempfile::tempdir().unwrap();
        let ar = archive(tmp.path());
        let root = tmp.path().join("out");

        let mut a = args(&ar);
        a.local = root.to_string_lossy().into_owned();

        let (logger, out) = make_logger();
        let empty = factory(vec![], Arc::new(Mutex::new(vec![])));
        run_restore(&a, &logger, &empty).unwrap();

        assert_eq!(fs::read_to_string(root.join("root.txt")).unwrap(), "root");
        assert!(out.contents().contains("restore complete"));
    }

    #[test]
    fn unknown_server_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let ar = archive(tmp.path());
        let cfg = config_file(tmp.path(), "/data");

        let mut a = args(&ar);
        a.config = cfg.to_string_lossy().into_owned();
        a.server = "missing".to_string();

        let (logger, _) = make_logger();
        let empty = factory(vec![], Arc::new(Mutex::new(vec![])));
        assert!(run_restore(&a, &logger, &empty).is_err());
    }

    #[test]
    fn remote_requires_force_when_non_interactive() {
        let tmp = tempfile::tempdir().unwrap();
        let ar = archive(tmp.path());
        let cfg = config_file(tmp.path(), "/data");

        let mut a = args(&ar);
        a.config = cfg.to_string_lossy().into_owned();
        a.server = "test".to_string();

        let (logger, _) = make_logger();
        let uploads = Arc::new(Mutex::new(vec![]));
        let f = factory(vec![], uploads.clone());

        assert!(run_restore(&a, &logger, &f).is_err());
        assert!(uploads.lock().unwrap().is_empty());
    }

    #[test]
    fn remote_dry_run_and_force_upload() {
        let tmp = tempfile::tempdir().unwrap();
        let ar = archive(tmp.path());
        let cfg = config_file(tmp.path(), "/data");

        let existing = FileInfo {
            path: "root.txt".to_string(),
            size: 1,
            mod_time: SystemTime::now(),
            is_dir: false,
        };

        // Dry run: no uploads, reports plan, skips existing root.txt.
        let mut a = args(&ar);
        a.config = cfg.to_string_lossy().into_owned();
        a.server = "test".to_string();
        a.dry_run = true;

        let (logger, out) = make_logger();
        let uploads = Arc::new(Mutex::new(vec![]));
        let f = factory(vec![existing.clone()], uploads.clone());
        run_restore(&a, &logger, &f).unwrap();
        assert!(uploads.lock().unwrap().is_empty());
        assert!(out.contents().contains("restore plan"));

        // Forced restore: uploads only the non-conflicting file.
        let mut a = args(&ar);
        a.config = cfg.to_string_lossy().into_owned();
        a.server = "test".to_string();
        a.force = true;

        let (logger, _) = make_logger();
        let f = factory(vec![existing], uploads.clone());
        run_restore(&a, &logger, &f).unwrap();

        let uploaded = uploads.lock().unwrap().clone();
        assert_eq!(uploaded, vec!["nested/child.txt".to_string()]);
    }

    #[test]
    fn missing_archive_errors_before_writes() {
        let tmp = tempfile::tempdir().unwrap();
        let mut a = args(&tmp.path().join("missing.tar.gz"));
        a.local = tmp.path().join("out").to_string_lossy().into_owned();

        let (logger, _) = make_logger();
        let empty = factory(vec![], Arc::new(Mutex::new(vec![])));
        assert!(run_restore(&a, &logger, &empty).is_err());
        assert!(!tmp.path().join("out").exists());
    }
}
