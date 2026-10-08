//! Remote restore executor.

use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;

use crate::connector::Connector;
use crate::log::Logger;

use super::archive_index::visit_archive;
use super::executor_local::{next_occurrence, restore_map};
use super::types::{EntryKind, ExecutionSummary, Plan};

/// Uploads planned entries to the connected remote target.
///
/// The connector must already be connected and have had its file listing used
/// for conflict detection. Dry runs perform no uploads.
pub fn execute_remote(
    conn: &mut dyn Connector,
    archive_path: &Path,
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
    let mut occurrences: HashMap<String, usize> = HashMap::new();
    let mut restored = 0usize;
    let mut upload_failures = 0usize;

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

        match conn.upload(reader, &entry.dest_path) {
            Ok(()) => {
                restored += 1;
                logger.info(format!("[green]restored[/green] {}", entry.dest_path));
            }
            Err(err) => {
                upload_failures += 1;
                logger.error(format!("[red]failed[/red] {}: {err}", entry.dest_path));
            }
        }

        Ok(())
    })?;

    super::log_plan(plan, logger, false);

    Ok(ExecutionSummary {
        restored,
        skipped: plan_summary.skipped(),
        failed: plan_summary.failed() + upload_failures,
        unsupported: plan_summary.fail_unsupported,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connector::FileInfo;
    use crate::restore::planner::build_plan;
    use crate::restore::types::{Decision, RestoreOptions};
    use anyhow::{bail, Result};
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::fs::File;
    use std::io::{self, Read, Write};
    use std::sync::{Arc, Mutex};
    use tar::{Builder, Header};

    struct MockConnector {
        uploads: Arc<Mutex<Vec<String>>>,
        fail_on: Option<String>,
    }

    impl Connector for MockConnector {
        fn connect(&mut self) -> Result<()> {
            Ok(())
        }
        fn list(&mut self) -> Result<Vec<FileInfo>> {
            Ok(vec![])
        }
        fn download(&mut self, _remote_path: &str, _w: &mut dyn Write) -> Result<()> {
            Ok(())
        }
        fn upload(&mut self, r: &mut dyn Read, remote_path: &str) -> Result<()> {
            if self.fail_on.as_deref() == Some(remote_path) {
                bail!("upload failed");
            }
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

    fn build_archive(dir: &Path) -> std::path::PathBuf {
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

        builder.finish().unwrap();
        let encoder = builder.into_inner().unwrap();
        encoder.finish().unwrap();
        dest
    }

    fn empty_logger() -> Logger {
        Logger::with_writers(Box::new(io::sink()), Box::new(io::sink()))
    }

    fn plan_for(archive: &Path, existing: &[&str]) -> Plan {
        let entries = crate::restore::scan_archive(archive).unwrap();
        let set: std::collections::HashSet<String> =
            existing.iter().map(|s| s.to_string()).collect();
        build_plan(&entries, &RestoreOptions::default(), &move |dest| {
            set.contains(dest)
        })
    }

    #[test]
    fn uploads_restorable_entries() {
        let tmp = tempfile::tempdir().unwrap();
        let archive = build_archive(tmp.path());
        let plan = plan_for(&archive, &[]);

        let uploads = Arc::new(Mutex::new(Vec::new()));
        let mut conn = MockConnector {
            uploads: uploads.clone(),
            fail_on: None,
        };

        let summary = execute_remote(&mut conn, &archive, &plan, false, &empty_logger()).unwrap();
        assert_eq!(summary.restored, 2);
        assert_eq!(summary.failed, 0);

        let mut uploaded = uploads.lock().unwrap().clone();
        uploaded.sort();
        assert_eq!(uploaded, vec!["nested/child.txt", "root.txt"]);
    }

    #[test]
    fn dry_run_does_not_upload() {
        let tmp = tempfile::tempdir().unwrap();
        let archive = build_archive(tmp.path());
        let plan = plan_for(&archive, &[]);

        let uploads = Arc::new(Mutex::new(Vec::new()));
        let mut conn = MockConnector {
            uploads: uploads.clone(),
            fail_on: None,
        };

        let summary = execute_remote(&mut conn, &archive, &plan, true, &empty_logger()).unwrap();
        assert_eq!(summary.restored, 2);
        assert!(uploads.lock().unwrap().is_empty());
    }

    #[test]
    fn skips_existing_without_overwrite() {
        let tmp = tempfile::tempdir().unwrap();
        let archive = build_archive(tmp.path());
        let plan = plan_for(&archive, &["root.txt"]);

        let uploads = Arc::new(Mutex::new(Vec::new()));
        let mut conn = MockConnector {
            uploads: uploads.clone(),
            fail_on: None,
        };

        let summary = execute_remote(&mut conn, &archive, &plan, false, &empty_logger()).unwrap();
        assert_eq!(summary.restored, 1);
        assert_eq!(summary.skipped, 1);

        let uploaded = uploads.lock().unwrap().clone();
        assert_eq!(uploaded, vec!["nested/child.txt"]);
    }

    #[test]
    fn continues_after_upload_failure() {
        let tmp = tempfile::tempdir().unwrap();
        let archive = build_archive(tmp.path());
        let plan = plan_for(&archive, &[]);

        let uploads = Arc::new(Mutex::new(Vec::new()));
        let mut conn = MockConnector {
            uploads: uploads.clone(),
            fail_on: Some("root.txt".to_string()),
        };

        let summary = execute_remote(&mut conn, &archive, &plan, false, &empty_logger()).unwrap();
        assert_eq!(summary.restored, 1);
        assert_eq!(summary.failed, 1);
        assert!(summary.has_failures());
    }

    #[test]
    fn unsupported_link_counts_toward_summary() {
        let tmp = tempfile::tempdir().unwrap();
        let archive = build_archive(tmp.path());
        let entries = crate::restore::scan_archive(&archive).unwrap();
        let mut plan = build_plan(&entries, &RestoreOptions::default(), &|_| false);
        plan.entries.push(crate::restore::types::PlanEntry {
            archive_path: "link".to_string(),
            occurrence: 0,
            dest_path: "link".to_string(),
            size: 0,
            kind: EntryKind::Symlink,
            decision: Decision::FailUnsupported,
            reason: "unsupported entry type: symlink".to_string(),
        });

        let uploads = Arc::new(Mutex::new(Vec::new()));
        let mut conn = MockConnector {
            uploads,
            fail_on: None,
        };
        let summary = execute_remote(&mut conn, &archive, &plan, false, &empty_logger()).unwrap();
        assert_eq!(summary.unsupported, 1);
        assert!(summary.has_failures());
    }
}
