//! Restore command support: archive indexing, planning, conflict detection and
//! execution.
//!
//! The flow is deliberately plan-first and safety-first:
//!
//! 1. Validate and index the `.tar.gz` archive.
//! 2. Build an immutable [`Plan`] with per-entry decisions.
//! 3. In dry-run mode, report the plan and write nothing.
//! 4. Otherwise execute the plan, continuing past per-file failures.

pub mod archive_index;
pub mod conflict_detector;
pub mod executor_local;
pub mod executor_remote;
pub mod path_rewrite;
pub mod planner;
pub mod types;

use std::path::Path;

use anyhow::{Context, Result};

use crate::log::Logger;

pub use archive_index::{scan_archive, visit_archive, ArchiveEntry};
pub use conflict_detector::ConflictSet;
pub use executor_local::execute_local;
pub use executor_remote::execute_remote;
pub use path_rewrite::{apply_strip_components, sanitize_archive_path, PathError};
pub use planner::build_plan;
pub use types::{
    Decision, EntryKind, ExecutionSummary, Plan, PlanEntry, PlanSummary, RestoreMode,
    RestoreOptions,
};

/// Validates that `path` is a readable `.tar.gz` archive.
pub fn validate_archive(path: &Path) -> Result<()> {
    scan_archive(path)
        .map(|_| ())
        .with_context(|| format!("invalid archive {}", path.display()))
}

/// Emits plan decisions through the logger.
///
/// Restore entries are only logged in dry-run mode; during a real run they are
/// logged as they are executed. Skip/fail decisions are always reported.
pub(crate) fn log_plan(plan: &Plan, logger: &Logger, dry_run: bool) {
    if !plan.duplicates.is_empty() {
        logger.warn(format!(
            "duplicate paths in archive (last wins): {}",
            plan.duplicates.join(", ")
        ));
    }

    for entry in &plan.entries {
        match entry.decision {
            Decision::Restore => {
                if dry_run {
                    logger.info(format!(
                        "[cyan]would restore[/cyan] {} ({} bytes)",
                        entry.dest_path, entry.size
                    ));
                }
            }
            Decision::SkipConflict => logger.info(format!(
                "[yellow]skip[/yellow] {} ({})",
                entry.archive_path, entry.reason
            )),
            Decision::SkipFilter => {
                logger.debug(format!("[dim]skip[/dim] {} (filtered)", entry.archive_path))
            }
            Decision::FailUnsupported => logger.error(format!(
                "[red]unsupported[/red] {} ({})",
                entry.archive_path, entry.reason
            )),
            Decision::FailInvalidPath => logger.error(format!(
                "[red]invalid path[/red] {} ({})",
                entry.archive_path, entry.reason
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::BufferWriter;

    #[test]
    fn validate_rejects_missing_archive() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(validate_archive(&tmp.path().join("nope.tar.gz")).is_err());
    }

    #[test]
    fn log_plan_reports_decisions() {
        let out = BufferWriter::new();
        let logger = Logger::with_writers(Box::new(out.clone()), Box::new(out.clone()));

        let plan = Plan {
            entries: vec![
                PlanEntry {
                    archive_path: "a.txt".to_string(),
                    occurrence: 0,
                    dest_path: "a.txt".to_string(),
                    size: 1,
                    kind: EntryKind::File,
                    decision: Decision::Restore,
                    reason: String::new(),
                },
                PlanEntry {
                    archive_path: "b.txt".to_string(),
                    occurrence: 0,
                    dest_path: "b.txt".to_string(),
                    size: 1,
                    kind: EntryKind::File,
                    decision: Decision::SkipConflict,
                    reason: "destination exists (use --overwrite)".to_string(),
                },
            ],
            duplicates: vec![],
        };

        log_plan(&plan, &logger, true);
        let text = out.contents();
        assert!(text.contains("would restore a.txt"));
        assert!(text.contains("skip b.txt"));
    }
}
