//! Include/exclude pattern matching.
//!
//! Globs follow shell-style rules: `*` and `?` never match a path separator,
//! `[...]` denotes a character class, and `\` escapes the next character.

/// Returns `true` when `path` matches the include patterns and does not match
/// the exclude patterns.
pub fn matches_patterns(path: &str, include: &[String], exclude: &[String]) -> bool {
    // Default include to ["*"].
    let default_include = vec!["*".to_string()];
    let include = if include.is_empty() {
        &default_include
    } else {
        include
    };

    // Check excludes first.
    for pattern in exclude {
        if let Some(dir) = pattern.strip_suffix('/') {
            if path.starts_with(&format!("{dir}/"))
                || path.starts_with(&format!("{dir}\\"))
                || path == dir
            {
                return false;
            }
        } else {
            let base = basename(path);
            if glob_match(pattern, &base) || glob_match(pattern, path) {
                return false;
            }
        }
    }

    // Check includes.
    for pattern in include {
        if pattern == "*" {
            return true;
        }
        let base = basename(path);
        if glob_match(pattern, &base) || glob_match(pattern, path) {
            return true;
        }
    }

    false
}

/// Returns the last element of a POSIX-style path.
fn basename(path: &str) -> String {
    if path.is_empty() {
        return ".".to_string();
    }
    let trimmed = path.trim_end_matches('/');
    if trimmed.is_empty() {
        return "/".to_string();
    }
    match trimmed.rsplit_once('/') {
        Some((_, base)) => base.to_string(),
        None => trimmed.to_string(),
    }
}

/// Matches a single glob pattern against a name.
fn glob_match(pattern: &str, name: &str) -> bool {
    let pattern: Vec<char> = pattern.chars().collect();
    let name: Vec<char> = name.chars().collect();
    match_here(&pattern, &name)
}

fn match_here(pattern: &[char], name: &[char]) -> bool {
    if pattern.is_empty() {
        return name.is_empty();
    }

    match pattern[0] {
        '*' => {
            if match_here(&pattern[1..], name) {
                return true;
            }
            if !name.is_empty() && name[0] != '/' {
                return match_here(pattern, &name[1..]);
            }
            false
        }
        '?' => !name.is_empty() && name[0] != '/' && match_here(&pattern[1..], &name[1..]),
        '[' => {
            if name.is_empty() || name[0] == '/' {
                return false;
            }
            match match_class(pattern, name[0]) {
                Some((matched, consumed)) => {
                    matched && match_here(&pattern[consumed..], &name[1..])
                }
                None => false,
            }
        }
        '\\' => {
            if pattern.len() >= 2 {
                !name.is_empty() && name[0] == pattern[1] && match_here(&pattern[2..], &name[1..])
            } else {
                false
            }
        }
        c => !name.is_empty() && name[0] == c && match_here(&pattern[1..], &name[1..]),
    }
}

/// Matches a `[...]` character class at the start of `pattern`.
///
/// Returns the match result and the number of pattern characters consumed
/// (including the closing `]`), or `None` when the class is malformed.
fn match_class(pattern: &[char], ch: char) -> Option<(bool, usize)> {
    debug_assert_eq!(pattern.first(), Some(&'['));

    let mut i = 1;
    let negate = pattern.get(i) == Some(&'^');
    if negate {
        i += 1;
    }

    let mut matched = false;
    let mut consumed_any = false;

    while i < pattern.len() {
        if pattern[i] == ']' && consumed_any {
            return Some((matched ^ negate, i + 1));
        }

        // Character range `a-z`.
        if i + 2 < pattern.len() && pattern[i + 1] == '-' && pattern[i + 2] != ']' {
            let (lo, hi) = (pattern[i], pattern[i + 2]);
            if ch >= lo && ch <= hi {
                matched = true;
            }
            i += 3;
            consumed_any = true;
            continue;
        }

        // Escaped character.
        let current = if pattern[i] == '\\' && i + 1 < pattern.len() {
            i += 1;
            pattern[i]
        } else {
            pattern[i]
        };
        if ch == current {
            matched = true;
        }
        i += 1;
        consumed_any = true;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pats(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn matches_patterns_cases() {
        struct Case<'a> {
            path: &'a str,
            include: &'a [&'a str],
            exclude: &'a [&'a str],
            expected: bool,
        }

        let cases = [
            Case {
                path: "file.txt",
                include: &["*"],
                exclude: &[],
                expected: true,
            },
            Case {
                path: "file.txt",
                include: &["*"],
                exclude: &["*.log"],
                expected: true,
            },
            Case {
                path: "debug.log",
                include: &["*"],
                exclude: &["*.log"],
                expected: false,
            },
            Case {
                path: "Logs/debug.log",
                include: &["*"],
                exclude: &["Logs/"],
                expected: false,
            },
            Case {
                path: "saves/game.sav",
                include: &["*"],
                exclude: &["Logs/"],
                expected: true,
            },
            Case {
                path: "temp.tmp",
                include: &["*"],
                exclude: &["*.log", "*.tmp"],
                expected: false,
            },
            Case {
                path: "game.sav",
                include: &["*.sav"],
                exclude: &[],
                expected: true,
            },
            Case {
                path: "config.ini",
                include: &["*.sav"],
                exclude: &[],
                expected: false,
            },
        ];

        for case in cases {
            let include = pats(case.include);
            let exclude = pats(case.exclude);
            assert_eq!(
                matches_patterns(case.path, &include, &exclude),
                case.expected,
                "path={} include={:?} exclude={:?}",
                case.path,
                case.include,
                case.exclude
            );
        }
    }

    #[test]
    fn star_does_not_cross_separator() {
        assert!(glob_match("*.log", "debug.log"));
        assert!(!glob_match("*.log", "Logs/debug.log"));
        assert!(glob_match("Logs/*", "Logs/debug.log"));
    }
}
