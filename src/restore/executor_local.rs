//! Local filesystem restore executor.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io;
use std::path::Path;

use anyhow::{Context, Result};

use crate::log::Logger;

use super::archive_index::visit_archive;
use super::types::{Decision, EntryKind, ExecutionSummary, Plan, PlanEntry};

/// Extracts planned entries from the archive into `root`.
///
/// Dry runs emit the plan without touching the filesystem. Per-file failures
/// are reported and counted, and do not stop the remaining entries.
pub fn execute_local(
    archive_path: &Path,
    root: &Path,
    plan: &Plan,
    dry_run: bool,
    logger: &Logger,
) -> Result<ExecutionSummary> {
    let plan_summary = plan.summary();

    if dry_run {
        super::log_plan(plan, logger, true);
        return Ok(ExecutionSummary {
            restored: plan_summary.restore,
            skipped: plan_summary.skipped(),
            failed: plan_summary.failed(),
            unsupported: plan_summary.fail_unsupported,
        });
    }

    let restore_map = restore_map(plan);

    fs::create_dir_all(root)
        .with_context(|| format!("failed to create destination {}", root.display()))?;

    let mut occurrences: HashMap<String, usize> = HashMap::new();
    let mut restored = 0usize;
    let mut write_failures = 0usize;

    visit_archive(archive_path, |path, kind, _size, reader| {
        let occurrence = next_occurrence(&mut occurrences, path);

        if kind != EntryKind::File {
            return Ok(());
        }

        let Some(entry) = restore_map.get(path).copied() else {
            return Ok(());
        };
        if entry.occurrence != occurrence {
            return Ok(());
        }

        let dest = root.join(&entry.dest_path);
        if let Some(parent) = dest.parent() {
            if let Err(err) = fs::create_dir_all(parent) {
                write_failures += 1;
                logger.error(format!("[red]failed[/red] {}: {err}", entry.dest_path));
                return Ok(());
            }
        }

        match File::create(&dest).and_then(|mut file| io::copy(reader, &mut file).map(|_| ())) {
            Ok(()) => {
                restored += 1;
                logger.info(format!("[green]restored[/green] {}", entry.dest_path));
            }
            Err(err) => {
                write_failures += 1;
                logger.error(format!("[red]failed[/red] {}: {err}", entry.dest_path));
            }
        }

        Ok(())
    })?;

    super::log_plan(plan, logger, false);

    Ok(ExecutionSummary {
        restored,
        skipped: plan_summary.skipped(),
        failed: plan_summary.failed() + write_failures,
        unsupported: plan_summary.fail_unsupported,
    })
}

pub(super) fn restore_map(plan: &Plan) -> HashMap<&str, &PlanEntry> {
    plan.entries
        .iter()
        .filter(|entry| entry.decision == Decision::Restore)
        .map(|entry| (entry.archive_path.as_str(), entry))
        .collect()
}

pub(super) fn next_occurrence(occurrences: &mut HashMap<String, usize>, path: &str) -> usize {
    let counter = occurrences.entry(path.to_string()).or_insert(0);
    let value = *counter;
    *counter += 1;
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backup::archive::create_archive;
    use crate::restore::planner::build_plan;
    use crate::restore::types::RestoreOptions;
    use flate2::read::GzDecoder;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use tar::{Builder, Header};

    fn build_archive(dir: &Path, symlink: bool) -> std::path::PathBuf {
        let dest = dir.join("archive.tar.gz");
        let file = File::create(&dest).unwrap();
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

        if symlink {
            let mut header = Header::new_gnu();
            header.set_entry_type(tar::EntryType::Symlink);
            header.set_size(0);
            header.set_mode(0o777);
            header.set_link_name("root.txt").unwrap();
            header.set_cksum();
            builder
                .append_data(&mut header, "link.txt", io::empty())
                .unwrap();
        }

        builder.finish().unwrap();
        let encoder = builder.into_inner().unwrap();
        encoder.finish().unwrap();
        dest
    }

    fn empty_logger() -> Logger {
        Logger::with_writers(Box::new(io::sink()), Box::new(io::sink()))
    }

    #[test]
    fn extracts_files() {
        let tmp = tempfile::tempdir().unwrap();
        let archive = build_archive(tmp.path(), false);
        let root = tmp.path().join("out");

        let entries = crate::restore::scan_archive(&archive).unwrap();
        let plan = build_plan(&entries, &RestoreOptions::default(), &|_| false);

        let summary = execute_local(&archive, &root, &plan, false, &empty_logger()).unwrap();

        assert_eq!(summary.restored, 2);
        assert_eq!(summary.failed, 0);
        assert_eq!(fs::read_to_string(root.join("root.txt")).unwrap(), "root");
        assert_eq!(
            fs::read_to_string(root.join("nested/child.txt")).unwrap(),
            "child"
        );
    }

    #[test]
    fn dry_run_writes_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let archive = build_archive(tmp.path(), false);
        let root = tmp.path().join("out");

        let entries = crate::restore::scan_archive(&archive).unwrap();
        let plan = build_plan(&entries, &RestoreOptions::default(), &|_| false);

        let summary = execute_local(&archive, &root, &plan, true, &empty_logger()).unwrap();

        assert_eq!(summary.restored, 2);
        assert!(!root.exists(), "dry-run must not create the destination");
    }

    #[test]
    fn overwrite_policy() {
        let tmp = tempfile::tempdir().unwrap();
        let archive = build_archive(tmp.path(), false);
        let root = tmp.path().join("out");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("root.txt"), "old").unwrap();

        let entries = crate::restore::scan_archive(&archive).unwrap();

        // Without overwrite: skip existing, restore the new file.
        let plan = build_plan(&entries, &RestoreOptions::default(), &|dest| {
            root.join(dest).exists()
        });
        let summary = execute_local(&archive, &root, &plan, false, &empty_logger()).unwrap();
        assert_eq!(summary.restored, 1);
        assert_eq!(summary.skipped, 1);
        assert_eq!(fs::read_to_string(root.join("root.txt")).unwrap(), "old");

        // With overwrite: replace existing.
        let options = RestoreOptions {
            overwrite: true,
            ..Default::default()
        };
        let plan = build_plan(&entries, &options, &|dest| root.join(dest).exists());
        let summary = execute_local(&archive, &root, &plan, false, &empty_logger()).unwrap();
        assert_eq!(summary.restored, 2);
        assert_eq!(fs::read_to_string(root.join("root.txt")).unwrap(), "root");
    }

    #[test]
    fn unsupported_links_count_as_failed() {
        let tmp = tempfile::tempdir().unwrap();
        let archive = build_archive(tmp.path(), true);
        let root = tmp.path().join("out");

        let entries = crate::restore::scan_archive(&archive).unwrap();
        let plan = build_plan(&entries, &RestoreOptions::default(), &|_| false);

        let summary = execute_local(&archive, &root, &plan, false, &empty_logger()).unwrap();
        assert_eq!(summary.restored, 2);
        assert_eq!(summary.unsupported, 1);
        assert!(summary.has_failures());
        assert!(!root.join("link.txt").exists());
    }

    #[test]
    fn reads_archive_created_by_backup() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("a.txt"), "a").unwrap();
        let archive = tmp.path().join("a.tar.gz");
        create_archive(&src, &archive).unwrap();

        let entries = crate::restore::scan_archive(&archive).unwrap();
        let plan: Plan = build_plan(&entries, &RestoreOptions::default(), &|_| false);
        assert_eq!(plan.summary().restore, 1);

        // Sanity check the archive can be decoded with the gzip reader.
        let _ = GzDecoder::new(File::open(&archive).unwrap());
    }
}
