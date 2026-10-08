//! `.tar.gz` archive reading and indexing.

use std::fs::File;
use std::io::Read;
use std::path::Path;

use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use tar::{Archive, EntryType};

use super::types::EntryKind;

/// Metadata for a single archive member.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveEntry {
    /// Normalized (slash-separated, no trailing separator) archive path.
    pub path: String,
    /// Size in bytes.
    pub size: u64,
    /// Entry kind.
    pub kind: EntryKind,
}

/// Normalizes a raw archive path for consistent matching.
pub fn normalize_path(raw: &str) -> String {
    raw.replace('\\', "/").trim_end_matches('/').to_string()
}

fn kind_from(entry_type: EntryType) -> EntryKind {
    match entry_type {
        EntryType::Regular | EntryType::Continuous => EntryKind::File,
        EntryType::Directory => EntryKind::Directory,
        EntryType::Symlink => EntryKind::Symlink,
        EntryType::Link => EntryKind::Hardlink,
        _ => EntryKind::Other,
    }
}

/// Walks every archive member, invoking `visit` with the normalized path,
/// entry kind, size and a reader for the entry's content.
///
/// Nothing is written to disk; entries that the visitor does not read are
/// skipped by the tar reader.
pub fn visit_archive<F>(path: &Path, mut visit: F) -> Result<()>
where
    F: FnMut(&str, EntryKind, u64, &mut dyn Read) -> Result<()>,
{
    let file =
        File::open(path).with_context(|| format!("failed to open archive {}", path.display()))?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);

    let entries = archive
        .entries()
        .context("failed to read archive (is it a valid .tar.gz?)")?;

    for entry in entries {
        let mut entry = entry.context("failed to read archive entry")?;
        let kind = kind_from(entry.header().entry_type());
        let raw = entry
            .path()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();
        let normalized = normalize_path(&raw);
        let size = entry.size();

        visit(&normalized, kind, size, &mut entry)?;
    }

    Ok(())
}

/// Indexes every archive member in deterministic order without reading content.
pub fn scan_archive(path: &Path) -> Result<Vec<ArchiveEntry>> {
    let mut entries = Vec::new();
    visit_archive(path, |entry_path, kind, size, _reader| {
        entries.push(ArchiveEntry {
            path: entry_path.to_string(),
            size,
            kind,
        });
        Ok(())
    })?;
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backup::archive::create_archive;
    use std::fs;
    use std::io::Write;

    fn sample_archive(dir: &Path) -> std::path::PathBuf {
        let src = dir.join("src");
        fs::create_dir_all(src.join("nested")).unwrap();
        fs::write(src.join("root.txt"), b"root").unwrap();
        fs::write(src.join("nested").join("child.txt"), b"child").unwrap();
        let dest = dir.join("out.tar.gz");
        create_archive(&src, &dest).unwrap();
        dest
    }

    #[test]
    fn indexes_valid_archive() {
        let tmp = tempfile::tempdir().unwrap();
        let archive = sample_archive(tmp.path());

        let entries = scan_archive(&archive).unwrap();
        let paths: Vec<String> = entries
            .iter()
            .filter(|e| e.kind == EntryKind::File)
            .map(|e| e.path.clone())
            .collect();

        assert!(paths.contains(&"root.txt".to_string()));
        assert!(paths.contains(&"nested/child.txt".to_string()));
    }

    #[test]
    fn rejects_missing_archive() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(scan_archive(&tmp.path().join("missing.tar.gz")).is_err());
    }

    #[test]
    fn rejects_non_gzip_file() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("bad.tar.gz");
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(b"this is not a tar.gz").unwrap();
        drop(file);
        assert!(scan_archive(&path).is_err());
    }

    #[test]
    fn visit_reads_file_content() {
        let tmp = tempfile::tempdir().unwrap();
        let archive = sample_archive(tmp.path());

        let mut content = String::new();
        visit_archive(&archive, |path, kind, _size, reader| {
            if kind == EntryKind::File && path == "root.txt" {
                reader.read_to_string(&mut content)?;
            }
            Ok(())
        })
        .unwrap();

        assert_eq!(content, "root");
    }
}
