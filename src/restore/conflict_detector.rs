//! Destination conflict detection.
//!
//! For remote restores, existing paths are derived from a successful connector
//! listing; a failed listing must abort the plan rather than risk a misleading
//! dry-run. For local restores, conflicts are resolved by checking the
//! filesystem directly (see [`super::executor_local`]).

use std::collections::HashSet;

use anyhow::{Context, Result};

use crate::connector::Connector;

/// Set of destination paths that already exist at the target.
#[derive(Debug, Clone, Default)]
pub struct ConflictSet {
    existing: HashSet<String>,
}

impl ConflictSet {
    /// Builds a conflict set from a collection of existing relative paths.
    pub fn from_paths(paths: impl IntoIterator<Item = String>) -> Self {
        Self {
            existing: paths.into_iter().collect(),
        }
    }

    /// Builds a conflict set by listing the connected remote target.
    ///
    /// Returns an error when the listing fails so that callers can abort the
    /// restore before performing any writes.
    pub fn from_remote(conn: &mut dyn Connector) -> Result<Self> {
        let files = conn
            .list()
            .context("failed to list remote files for conflict detection")?;
        Ok(Self::from_paths(files.into_iter().map(|file| file.path)))
    }

    /// Returns `true` if `dest` already exists.
    pub fn contains(&self, dest: &str) -> bool {
        self.existing.contains(dest)
    }

    /// Returns `true` if no destination paths exist.
    pub fn is_empty(&self) -> bool {
        self.existing.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connector::{Connector, FileInfo};
    use anyhow::{bail, Result};
    use std::io::{Read, Write};
    use std::time::SystemTime;

    struct ListConnector {
        files: Result<Vec<FileInfo>>,
    }

    impl Connector for ListConnector {
        fn connect(&mut self) -> Result<()> {
            Ok(())
        }
        fn list(&mut self) -> Result<Vec<FileInfo>> {
            match &self.files {
                Ok(files) => Ok(files.clone()),
                Err(err) => bail!("{err}"),
            }
        }
        fn download(&mut self, _remote_path: &str, _w: &mut dyn Write) -> Result<()> {
            Ok(())
        }
        fn upload(&mut self, _r: &mut dyn Read, _remote_path: &str) -> Result<()> {
            Ok(())
        }
        fn close(&mut self) -> Result<()> {
            Ok(())
        }
        fn name(&self) -> String {
            "list".to_string()
        }
    }

    fn file(path: &str) -> FileInfo {
        FileInfo {
            path: path.to_string(),
            size: 1,
            mod_time: SystemTime::now(),
            is_dir: false,
        }
    }

    #[test]
    fn from_paths_contains() {
        let set = ConflictSet::from_paths(vec!["a.txt".to_string()]);
        assert!(set.contains("a.txt"));
        assert!(!set.contains("b.txt"));
        assert!(!set.is_empty());
    }

    #[test]
    fn from_remote_collects_paths() {
        let mut conn = ListConnector {
            files: Ok(vec![file("a.txt"), file("dir/b.txt")]),
        };
        let set = ConflictSet::from_remote(&mut conn).unwrap();
        assert!(set.contains("a.txt"));
        assert!(set.contains("dir/b.txt"));
    }

    #[test]
    fn from_remote_propagates_list_failure() {
        let mut conn = ListConnector {
            files: Err(anyhow::anyhow!("list failed")),
        };
        assert!(ConflictSet::from_remote(&mut conn).is_err());
    }
}
