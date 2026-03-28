//! Core linting logic — filesystem walk and violation collection.

use std::fmt;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::config::Config;
use crate::convention::Convention;

// ---------------------------------------------------------------------------
// Violation
// ---------------------------------------------------------------------------

/// A single naming violation detected during a lint run.
///
/// The [`fmt::Display`] output intentionally mirrors the `error[…]: …` format
/// used by `rustc` and `clippy`, making it easy to embed in CI logs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// Path to the offending file, relative to the scanned project root.
    pub path: PathBuf,
    /// The file stem that violated the convention (no extension, no directory).
    pub stem: String,
    /// The convention the stem was required to follow.
    pub expected: Convention,
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "error[convention]: `{}` — stem `{}` does not follow {} convention",
            self.path.display(),
            self.stem,
            self.expected,
        )
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Walk `project_root` according to `config` and return every naming
/// violation found.
///
/// **Search scope** — for each `(extension, convention)` pair in
/// `config.rules`:
///
/// * If `config.dirs` has an entry for that extension, only those directories
///   (resolved relative to `project_root`) are walked.
/// * Otherwise the entire `project_root` is walked recursively.
///
/// **Always skipped** — hidden entries (names beginning with `.`) and the
/// `target/` directory are never descended into.
///
/// A configured directory that does not exist on disk emits a warning to
/// `stderr` and is otherwise silently ignored.
///
/// # Examples
///
/// ```no_run
/// use convention_lint::{config::load_config, lint::run};
///
/// let cfg = load_config(std::path::Path::new("Cargo.toml")).unwrap();
/// let violations = run(&cfg, std::path::Path::new("."));
/// for v in &violations {
///     eprintln!("{v}");
/// }
/// ```
#[must_use]
pub fn run(config: &Config, project_root: &Path) -> Vec<Violation> {
    config
        .rules
        .iter()
        .flat_map(|(ext, convention)| {
            search_dirs(config, ext, project_root)
                .into_iter()
                .flat_map(|dir| walk_dir(&dir, ext, convention, project_root))
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Resolve the directories to search for `ext`, falling back to the project
/// root when no explicit list is configured.
fn search_dirs(config: &Config, ext: &str, project_root: &Path) -> Vec<PathBuf> {
    config.dirs.get(ext).map_or_else(
        || vec![project_root.to_path_buf()],
        |dirs| {
            dirs.iter()
                .map(|d| {
                    if d.as_os_str().is_empty() || d == Path::new(".") {
                        project_root.to_path_buf()
                    } else {
                        project_root.join(d)
                    }
                })
                .collect()
        },
    )
}

/// Recursively walk `dir`, yielding a [`Violation`] for every file whose
/// extension matches `ext` but whose stem does not satisfy `convention`.
fn walk_dir(dir: &Path, ext: &str, convention: &Convention, project_root: &Path) -> Vec<Violation> {
    if !dir.exists() {
        eprintln!(
            "warning: directory `{}` for extension `.{ext}` does not exist",
            dir.display()
        );
        return Vec::new();
    }

    WalkDir::new(dir)
        .into_iter()
        .filter_entry(|e| {
            // Always descend into the root itself — its name is irrelevant and
            // may start with `.` (e.g. temporary directories used in tests).
            if e.depth() == 0 {
                return true;
            }
            let name = e.file_name().to_string_lossy();
            !name.starts_with('.') && name != "target"
        })
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some(ext))
        .filter_map(|e| {
            let stem = e.path().file_stem()?.to_str()?.to_owned();
            if convention.is_valid(&stem) {
                return None;
            }
            let rel = e
                .path()
                .strip_prefix(project_root)
                .unwrap_or_else(|_| e.path())
                .to_path_buf();
            Some(Violation {
                path: rel,
                stem,
                expected: convention.clone(),
            })
        })
        .collect()
}
