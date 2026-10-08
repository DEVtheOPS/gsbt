//! Restore planner: filtering, path rewriting and decision assignment.

use std::collections::HashMap;

use crate::connector::matcher::matches_patterns;

use super::archive_index::ArchiveEntry;
use super::path_rewrite::{apply_strip_components, sanitize_archive_path};
use super::types::{Decision, EntryKind, Plan, PlanEntry, RestoreOptions};

/// Builds an immutable restore plan.
///
/// `exists` reports whether a destination path already exists, which drives
/// conflict detection. Directory entries are ignored (parent directories are
/// created on demand during execution).
pub fn build_plan(
    entries: &[ArchiveEntry],
    options: &RestoreOptions,
    exists: &dyn Fn(&str) -> bool,
) -> Plan {
    let mut occurrences: HashMap<String, usize> = HashMap::new();
    let mut planned: Vec<PlanEntry> = Vec::new();

    for entry in entries {
        if entry.kind == EntryKind::Directory {
            continue;
        }

        let occurrence = {
            let counter = occurrences.entry(entry.path.clone()).or_insert(0);
            let value = *counter;
            *counter += 1;
            value
        };

        let planned_entry = match sanitize_archive_path(&entry.path) {
            Err(err) => PlanEntry {
                archive_path: entry.path.clone(),
                occurrence,
                dest_path: String::new(),
                size: entry.size,
                kind: entry.kind,
                decision: Decision::FailInvalidPath,
                reason: format!("invalid path: {err}"),
            },
            Ok(sanitized) => {
                if !matches_patterns(&sanitized, &options.includes, &options.excludes) {
                    PlanEntry {
                        archive_path: entry.path.clone(),
                        occurrence,
                        dest_path: sanitized,
                        size: entry.size,
                        kind: entry.kind,
                        decision: Decision::SkipFilter,
                        reason: "excluded by filter".to_string(),
                    }
                } else {
                    match apply_strip_components(&sanitized, options.strip_components) {
                        None => PlanEntry {
                            archive_path: entry.path.clone(),
                            occurrence,
                            dest_path: String::new(),
                            size: entry.size,
                            kind: entry.kind,
                            decision: Decision::FailInvalidPath,
                            reason: "strip-components removed all path segments".to_string(),
                        },
                        Some(dest) => {
                            if entry.kind != EntryKind::File {
                                PlanEntry {
                                    archive_path: entry.path.clone(),
                                    occurrence,
                                    dest_path: dest,
                                    size: entry.size,
                                    kind: entry.kind,
                                    decision: Decision::FailUnsupported,
                                    reason: format!(
                                        "unsupported entry type: {}",
                                        entry.kind.as_str()
                                    ),
                                }
                            } else if exists(&dest) && !options.overwrite {
                                PlanEntry {
                                    archive_path: entry.path.clone(),
                                    occurrence,
                                    dest_path: dest,
                                    size: entry.size,
                                    kind: entry.kind,
                                    decision: Decision::SkipConflict,
                                    reason: "destination exists (use --overwrite)".to_string(),
                                }
                            } else {
                                PlanEntry {
                                    archive_path: entry.path.clone(),
                                    occurrence,
                                    dest_path: dest,
                                    size: entry.size,
                                    kind: entry.kind,
                                    decision: Decision::Restore,
                                    reason: String::new(),
                                }
                            }
                        }
                    }
                }
            }
        };

        planned.push(planned_entry);
    }

    deduplicate(planned)
}

/// Keeps the last plan entry per destination path and records duplicates.
fn deduplicate(planned: Vec<PlanEntry>) -> Plan {
    let mut last_index: HashMap<String, usize> = HashMap::new();
    let mut counts: HashMap<String, usize> = HashMap::new();

    for (index, entry) in planned.iter().enumerate() {
        if !entry.dest_path.is_empty() {
            last_index.insert(entry.dest_path.clone(), index);
            *counts.entry(entry.dest_path.clone()).or_insert(0) += 1;
        }
    }

    let mut duplicates: Vec<String> = counts
        .iter()
        .filter(|(_, count)| **count > 1)
        .map(|(path, _)| path.clone())
        .collect();
    duplicates.sort();

    let entries = planned
        .into_iter()
        .enumerate()
        .filter(|(index, entry)| {
            entry.dest_path.is_empty() || last_index.get(&entry.dest_path) == Some(index)
        })
        .map(|(_, entry)| entry)
        .collect();

    Plan {
        entries,
        duplicates,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn file(path: &str, size: u64) -> ArchiveEntry {
        ArchiveEntry {
            path: path.to_string(),
            size,
            kind: EntryKind::File,
        }
    }

    fn plan(entries: &[ArchiveEntry], options: &RestoreOptions, existing: &[&str]) -> Plan {
        let set: HashSet<String> = existing.iter().map(|s| s.to_string()).collect();
        build_plan(entries, options, &move |dest| set.contains(dest))
    }

    #[test]
    fn plans_restore_for_new_files() {
        let entries = vec![file("a.txt", 1), file("nested/b.txt", 2)];
        let plan = plan(&entries, &RestoreOptions::default(), &[]);

        let summary = plan.summary();
        assert_eq!(summary.restore, 2);
        assert_eq!(summary.failed(), 0);
        assert_eq!(plan.entries.len(), 2);
    }

    #[test]
    fn include_exclude_precedence() {
        let entries = vec![
            file("game.sav", 1),
            file("debug.log", 2),
            file("config.ini", 3),
        ];
        let options = RestoreOptions {
            includes: vec!["*.sav".to_string(), "*.ini".to_string()],
            excludes: vec!["*.ini".to_string()],
            ..Default::default()
        };
        let plan = plan(&entries, &options, &[]);

        let restored: Vec<&str> = plan
            .entries
            .iter()
            .filter(|e| e.decision == Decision::Restore)
            .map(|e| e.dest_path.as_str())
            .collect();
        assert_eq!(restored, vec!["game.sav"]);

        let filtered: Vec<&str> = plan
            .entries
            .iter()
            .filter(|e| e.decision == Decision::SkipFilter)
            .map(|e| e.archive_path.as_str())
            .collect();
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn conflict_without_overwrite_is_skipped() {
        let entries = vec![file("a.txt", 1)];
        let plan = plan(&entries, &RestoreOptions::default(), &["a.txt"]);

        assert_eq!(plan.entries[0].decision, Decision::SkipConflict);
        assert_eq!(plan.summary().skipped(), 1);
    }

    #[test]
    fn conflict_with_overwrite_is_restored() {
        let entries = vec![file("a.txt", 1)];
        let options = RestoreOptions {
            overwrite: true,
            ..Default::default()
        };
        let plan = plan(&entries, &options, &["a.txt"]);
        assert_eq!(plan.entries[0].decision, Decision::Restore);
    }

    #[test]
    fn unsafe_paths_fail() {
        let entries = vec![file("../secret", 1), file("/etc/passwd", 2)];
        let plan = plan(&entries, &RestoreOptions::default(), &[]);

        assert!(plan
            .entries
            .iter()
            .all(|e| e.decision == Decision::FailInvalidPath));
        assert_eq!(plan.summary().failed(), 2);
    }

    #[test]
    fn links_are_unsupported() {
        let entries = vec![ArchiveEntry {
            path: "link".to_string(),
            size: 0,
            kind: EntryKind::Symlink,
        }];
        let plan = plan(&entries, &RestoreOptions::default(), &[]);

        assert_eq!(plan.entries[0].decision, Decision::FailUnsupported);
        assert_eq!(plan.summary().fail_unsupported, 1);
    }

    #[test]
    fn strip_components_rewrites_dest() {
        let entries = vec![file("backup/saves/a.txt", 1)];
        let options = RestoreOptions {
            strip_components: 2,
            ..Default::default()
        };
        let plan = plan(&entries, &options, &[]);
        assert_eq!(plan.entries[0].dest_path, "a.txt");
    }

    #[test]
    fn over_stripping_fails() {
        let entries = vec![file("a.txt", 1)];
        let options = RestoreOptions {
            strip_components: 1,
            ..Default::default()
        };
        let plan = plan(&entries, &options, &[]);
        assert_eq!(plan.entries[0].decision, Decision::FailInvalidPath);
    }

    #[test]
    fn duplicate_destinations_last_wins() {
        let entries = vec![file("a.txt", 1), file("a.txt", 2)];
        let plan = plan(&entries, &RestoreOptions::default(), &[]);

        assert_eq!(plan.entries.len(), 1);
        assert_eq!(plan.entries[0].size, 2);
        assert_eq!(plan.duplicates, vec!["a.txt".to_string()]);
    }

    #[test]
    fn empty_after_filter_is_empty_plan() {
        let entries = vec![file("a.txt", 1)];
        let options = RestoreOptions {
            includes: vec!["*.sav".to_string()],
            ..Default::default()
        };
        let plan = plan(&entries, &options, &[]);
        assert_eq!(plan.summary().restore, 0);
        assert_eq!(plan.summary().skip_filter, 1);
    }
}
