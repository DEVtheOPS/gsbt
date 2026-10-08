//! Archive path safety and rewriting.
//!
//! Rejects absolute paths and `..` traversal so that restored files can never
//! escape the target root, and applies `--strip-components` deterministically.

use std::fmt;

/// Reason a path was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathError {
    /// Path is absolute or has a Windows drive prefix.
    Absolute,
    /// Path contains a `..` component.
    Traversal,
    /// Path is empty or resolves to no components.
    Empty,
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            PathError::Absolute => "absolute paths are not allowed",
            PathError::Traversal => "path traversal ('..') is not allowed",
            PathError::Empty => "path is empty",
        })
    }
}

impl std::error::Error for PathError {}

/// Normalizes and validates an archive member path.
///
/// Backslashes are converted to forward slashes, `.` and empty components are
/// dropped, and absolute/traversal paths are rejected.
pub fn sanitize_archive_path(raw: &str) -> Result<String, PathError> {
    let replaced = raw.replace('\\', "/");

    if replaced.starts_with('/') {
        return Err(PathError::Absolute);
    }

    // Reject Windows drive prefixes such as `C:\...`.
    let bytes = replaced.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
        return Err(PathError::Absolute);
    }

    let mut components = Vec::new();
    for component in replaced.split('/') {
        match component {
            "" | "." => continue,
            ".." => return Err(PathError::Traversal),
            other => components.push(other),
        }
    }

    if components.is_empty() {
        return Err(PathError::Empty);
    }

    Ok(components.join("/"))
}

/// Removes the first `n` path components.
///
/// Returns `None` when stripping removes every component (the entry would map
/// to nothing).
pub fn apply_strip_components(path: &str, n: usize) -> Option<String> {
    if n == 0 {
        return Some(path.to_string());
    }

    let components: Vec<&str> = path.split('/').filter(|c| !c.is_empty()).collect();
    if n >= components.len() {
        return None;
    }

    Some(components[n..].join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_normal_paths() {
        assert_eq!(sanitize_archive_path("a/b/c.txt").unwrap(), "a/b/c.txt");
        assert_eq!(sanitize_archive_path("./a/b.txt").unwrap(), "a/b.txt");
        assert_eq!(sanitize_archive_path("a//b.txt").unwrap(), "a/b.txt");
        assert_eq!(sanitize_archive_path("a\\b.txt").unwrap(), "a/b.txt");
        assert_eq!(sanitize_archive_path("a/b/").unwrap(), "a/b");
    }

    #[test]
    fn rejects_unsafe_paths() {
        assert_eq!(
            sanitize_archive_path("/etc/passwd"),
            Err(PathError::Absolute)
        );
        assert_eq!(
            sanitize_archive_path("C:\\Windows\\system32"),
            Err(PathError::Absolute)
        );
        assert_eq!(
            sanitize_archive_path("../secret"),
            Err(PathError::Traversal)
        );
        assert_eq!(
            sanitize_archive_path("a/../../secret"),
            Err(PathError::Traversal)
        );
        assert_eq!(sanitize_archive_path(""), Err(PathError::Empty));
        assert_eq!(sanitize_archive_path("./"), Err(PathError::Empty));
    }

    #[test]
    fn strip_components_matrix() {
        assert_eq!(
            apply_strip_components("a/b/c.txt", 0),
            Some("a/b/c.txt".to_string())
        );
        assert_eq!(
            apply_strip_components("a/b/c.txt", 1),
            Some("b/c.txt".to_string())
        );
        assert_eq!(
            apply_strip_components("a/b/c.txt", 2),
            Some("c.txt".to_string())
        );
        assert_eq!(apply_strip_components("a/b/c.txt", 3), None);
        assert_eq!(apply_strip_components("a/b/c.txt", 4), None);
        assert_eq!(apply_strip_components("c.txt", 1), None);
    }
}
