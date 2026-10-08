//! `.tar.gz` archive creation.

use std::fs::{self, File};
use std::io;
use std::path::Path;

use anyhow::{bail, Context, Result};
use chrono::Utc;
use flate2::write::GzEncoder;
use flate2::Compression;
use tar::{Builder, Header};

/// Compresses the contents of `src_dir` into a `.tar.gz` at `dest_path`.
///
/// Archive paths are stored relative to `src_dir`.
pub fn create_archive(src_dir: &Path, dest_path: &Path) -> Result<()> {
    if src_dir.as_os_str().is_empty() {
        bail!("srcDir is required");
    }
    if dest_path.as_os_str().is_empty() {
        bail!("destPath is required");
    }

    if let Some(parent) = dest_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).context("failed to create archive directory")?;
        }
    }

    let file = File::create(dest_path).context("failed to create archive file")?;
    let encoder = GzEncoder::new(file, Compression::default());
    let mut builder = Builder::new(encoder);

    append_dir(&mut builder, src_dir, src_dir)?;

    builder.finish().context("failed to finalize archive")?;
    let encoder = builder.into_inner().context("failed to finalize gzip")?;
    encoder.finish().context("failed to finalize gzip")?;

    Ok(())
}

fn append_dir(builder: &mut Builder<GzEncoder<File>>, root: &Path, dir: &Path) -> Result<()> {
    let entries = fs::read_dir(dir).with_context(|| format!("read dir {}", dir.display()))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let rel = path
            .strip_prefix(root)
            .with_context(|| format!("strip prefix for {}", path.display()))?;
        let archive_name = to_archive_path(rel);
        let meta = entry
            .metadata()
            .with_context(|| format!("metadata for {}", path.display()))?;

        let mut header = Header::new_gnu();
        header.set_metadata(&meta);

        if meta.is_dir() {
            header.set_size(0);
            header.set_entry_type(tar::EntryType::Directory);
            builder.append_data(&mut header, &archive_name, io::empty())?;
            append_dir(builder, root, &path)?;
        } else if meta.is_file() {
            let mut file = File::open(&path).with_context(|| format!("open {}", path.display()))?;
            builder.append_data(&mut header, &archive_name, &mut file)?;
        } else if meta.file_type().is_symlink() {
            header.set_size(0);
            header.set_entry_type(tar::EntryType::Symlink);
            if let Ok(target) = fs::read_link(&path) {
                let _ = header.set_link_name(&target);
            }
            builder.append_data(&mut header, &archive_name, io::empty())?;
        }
    }

    Ok(())
}

fn to_archive_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// Returns a UTC timestamped filename in the gsbt format.
pub fn timestamped_filename() -> String {
    format!("{}.tar.gz", Utc::now().format("%Y-%m-%d_%H%M%S"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::read::GzDecoder;
    use std::collections::HashMap;
    use std::io::Read;
    use tar::Archive;

    #[test]
    fn creates_archive_with_expected_contents() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let src = tmp.path().join("src");
        fs::create_dir_all(src.join("nested")).expect("mkdir");
        fs::write(src.join("root.txt"), b"root").expect("write");
        fs::write(src.join("nested").join("child.txt"), b"child").expect("write");

        let dest = tmp.path().join("out.tar.gz");
        create_archive(&src, &dest).expect("create archive");

        let file = File::open(&dest).expect("open archive");
        let gz = GzDecoder::new(file);
        let mut archive = Archive::new(gz);

        let mut seen: HashMap<String, String> = HashMap::new();
        for entry in archive.entries().expect("entries") {
            let mut entry = entry.expect("entry");
            let path = entry.path().expect("path").to_string_lossy().into_owned();
            let mut data = String::new();
            entry.read_to_string(&mut data).expect("read");
            seen.insert(path, data);
        }

        assert_eq!(seen.get("root.txt").map(String::as_str), Some("root"));
        assert_eq!(
            seen.get("nested/child.txt").map(String::as_str),
            Some("child")
        );
    }

    #[test]
    fn timestamped_filename_format() {
        let name = timestamped_filename();
        assert!(name.ends_with(".tar.gz"));
        assert_eq!(name.len(), "2006-01-02_150405.tar.gz".len());
    }
}
