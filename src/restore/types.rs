//! Restore domain types.

/// Destination mode for a restore operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestoreMode {
    Remote,
    Local,
}

impl RestoreMode {
    pub fn as_str(self) -> &'static str {
        match self {
            RestoreMode::Remote => "remote",
            RestoreMode::Local => "local",
        }
    }
}

/// Kind of entry found in an archive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    File,
    Directory,
    Symlink,
    Hardlink,
    Other,
}

impl EntryKind {
    pub fn as_str(self) -> &'static str {
        match self {
            EntryKind::File => "file",
            EntryKind::Directory => "directory",
            EntryKind::Symlink => "symlink",
            EntryKind::Hardlink => "hardlink",
            EntryKind::Other => "other",
        }
    }
}

/// Per-entry restore decision produced by the planner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Entry will be restored.
    Restore,
    /// Entry exists at the destination and overwrite is disabled.
    SkipConflict,
    /// Entry was excluded by include/exclude filters.
    SkipFilter,
    /// Entry is a symlink/hardlink or other unsupported type.
    FailUnsupported,
    /// Entry has an unsafe path (absolute, traversal, over-stripped).
    FailInvalidPath,
}

impl Decision {
    pub fn as_str(self) -> &'static str {
        match self {
            Decision::Restore => "restore",
            Decision::SkipConflict => "skip_conflict",
            Decision::SkipFilter => "skip_filter",
            Decision::FailUnsupported => "fail_unsupported",
            Decision::FailInvalidPath => "fail_invalid_path",
        }
    }

    pub fn is_skip(self) -> bool {
        matches!(self, Decision::SkipConflict | Decision::SkipFilter)
    }

    pub fn is_fail(self) -> bool {
        matches!(self, Decision::FailUnsupported | Decision::FailInvalidPath)
    }
}

/// A single planned restore action.
#[derive(Debug, Clone)]
pub struct PlanEntry {
    /// Sanitized archive path (normalized, slash-separated).
    pub archive_path: String,
    /// Zero-based occurrence index of `archive_path` within the archive.
    pub occurrence: usize,
    /// Destination path relative to the target root (empty when invalid).
    pub dest_path: String,
    /// Size in bytes.
    pub size: u64,
    /// Entry kind.
    pub kind: EntryKind,
    /// Planned decision.
    pub decision: Decision,
    /// Human-readable reason for skip/fail decisions.
    pub reason: String,
}

/// Aggregated plan counts.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PlanSummary {
    pub restore: usize,
    pub skip_conflict: usize,
    pub skip_filter: usize,
    pub fail_unsupported: usize,
    pub fail_invalid_path: usize,
}

impl PlanSummary {
    /// Entries that will be skipped without being fatal.
    pub fn skipped(&self) -> usize {
        self.skip_conflict + self.skip_filter
    }

    /// Entries that count as failures.
    pub fn failed(&self) -> usize {
        self.fail_unsupported + self.fail_invalid_path
    }

    /// Whether the plan contains no restorable work.
    pub fn is_empty(&self) -> bool {
        self.restore == 0 && self.skipped() == 0 && self.failed() == 0
    }
}

/// An immutable restore plan.
#[derive(Debug, Clone, Default)]
pub struct Plan {
    pub entries: Vec<PlanEntry>,
    /// Destination paths that appeared more than once (last one wins).
    pub duplicates: Vec<String>,
}

impl Plan {
    /// Aggregates per-entry decisions into summary counts.
    pub fn summary(&self) -> PlanSummary {
        let mut summary = PlanSummary::default();
        for entry in &self.entries {
            match entry.decision {
                Decision::Restore => summary.restore += 1,
                Decision::SkipConflict => summary.skip_conflict += 1,
                Decision::SkipFilter => summary.skip_filter += 1,
                Decision::FailUnsupported => summary.fail_unsupported += 1,
                Decision::FailInvalidPath => summary.fail_invalid_path += 1,
            }
        }
        summary
    }
}

/// Options controlling planning and execution.
#[derive(Debug, Clone, Default)]
pub struct RestoreOptions {
    /// Overwrite existing destination files (default: skip).
    pub overwrite: bool,
    /// Include globs (empty means all).
    pub includes: Vec<String>,
    /// Exclude globs.
    pub excludes: Vec<String>,
    /// Number of leading path components to strip.
    pub strip_components: usize,
}

/// Result summary of a restore run.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ExecutionSummary {
    pub restored: usize,
    pub skipped: usize,
    pub failed: usize,
    pub unsupported: usize,
}

impl ExecutionSummary {
    /// Returns `true` when any entry was unsupported or otherwise failed.
    pub fn has_failures(&self) -> bool {
        self.failed > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(dest: &str, decision: Decision) -> PlanEntry {
        PlanEntry {
            archive_path: dest.to_string(),
            occurrence: 0,
            dest_path: dest.to_string(),
            size: 1,
            kind: EntryKind::File,
            decision,
            reason: String::new(),
        }
    }

    #[test]
    fn summary_aggregates_decisions() {
        let plan = Plan {
            entries: vec![
                entry("a", Decision::Restore),
                entry("b", Decision::SkipConflict),
                entry("c", Decision::SkipFilter),
                entry("d", Decision::FailUnsupported),
                entry("e", Decision::FailInvalidPath),
            ],
            duplicates: vec![],
        };

        let summary = plan.summary();
        assert_eq!(summary.restore, 1);
        assert_eq!(summary.skipped(), 2);
        assert_eq!(summary.failed(), 2);
        assert!(!summary.is_empty());
    }

    #[test]
    fn empty_summary_is_empty() {
        assert!(PlanSummary::default().is_empty());
    }

    #[test]
    fn decision_helpers() {
        assert!(Decision::SkipConflict.is_skip());
        assert!(Decision::SkipFilter.is_skip());
        assert!(!Decision::Restore.is_skip());
        assert!(Decision::FailUnsupported.is_fail());
        assert!(Decision::FailInvalidPath.is_fail());
    }
}
