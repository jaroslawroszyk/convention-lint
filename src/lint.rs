//! Core linting logic — filesystem walk and violation collection.

use crate::config::Config;
use crate::core::Convention;
use ignore::WalkBuilder;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

fn is_glob_pattern(s: &str) -> bool {
    s.contains('*') || s.contains('?') || s.contains('[')
}

/// Expand a dir entry that may contain glob metacharacters into concrete
/// directory paths.  When the pattern has no globs the path is returned
/// as-is (even if it doesn't exist yet — the walker will skip it).
fn resolve_dirs(dir: &Path, project_root: &Path) -> Vec<PathBuf> {
    let full_path = if dir.is_absolute() {
        dir.to_path_buf()
    } else {
        project_root.join(dir)
    };

    let full_path_str = full_path.to_string_lossy();

    if !is_glob_pattern(&full_path_str) {
        return vec![full_path];
    }

    let mut dirs = Vec::new();
    if let Ok(entries) = glob::glob(&full_path_str) {
        for entry in entries.flatten() {
            if entry.is_dir() {
                let file_name = entry
                    .file_name()
                    .map(|n| n.to_string_lossy())
                    .unwrap_or_default();

                if file_name.starts_with('.') || file_name == "target" {
                    continue;
                }

                dirs.push(entry);
            }
        }
    }
    dirs
}

/// A single naming violation detected during a lint run.
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

/// Walk `project_root` according to `config` and return every naming
/// violation found.
///
/// # Panics
///
/// This function will panic if the internal mutex is poisoned.
#[must_use]
pub fn run(config: &Config, project_root: &Path) -> Vec<Violation> {
    let violations = Arc::new(Mutex::new(Vec::new()));

    for rule in &config.rules {
        for dir in &rule.dirs {
            let targets = resolve_dirs(dir, project_root);

            for target in targets {
                if !target.exists() {
                    continue;
                }

                let violations_lock = Arc::clone(&violations);
                let matcher = rule.matcher.clone();
                let convention = rule.convention.clone();
                let root_owned = project_root.to_path_buf();

                let mut builder = WalkBuilder::new(target);
                builder.standard_filters(true).hidden(false).parents(false);

                if !rule.recursive {
                    builder.max_depth(Some(1));
                }

                let walker = builder.build_parallel();

                walker.run(|| {
                    let v_inner = Arc::clone(&violations_lock);
                    let m_inner = matcher.clone();
                    let c_inner = convention.clone();
                    let r_inner = root_owned.clone();

                    Box::new(move |result| {
                        if let Ok(entry) = result {
                            let path = entry.path();

                            let rel_path = path.strip_prefix(&r_inner).unwrap_or(path);

                            let rel_path_str = rel_path.to_string_lossy();
                            let file_name = entry.file_name().to_string_lossy();

                            if entry.file_type().is_some_and(|ft| ft.is_dir())
                                && file_name == "target"
                                || (file_name.starts_with('.') && entry.depth() > 0)
                            {
                                return ignore::WalkState::Skip;
                            }
                            if entry.file_type().is_some_and(|f| f.is_file())
                                && m_inner.is_match(&rel_path_str)
                            {
                                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                                    if !c_inner.is_valid(stem) {
                                        v_inner.lock().expect("mutex poisoned").push(Violation {
                                            path: rel_path.to_path_buf(),
                                            stem: stem.to_owned(),
                                            expected: c_inner.clone(),
                                        });
                                    }
                                }
                            }
                        }
                        ignore::WalkState::Continue
                    })
                });
            }
        }
    }

    let final_lock = Arc::try_unwrap(violations).expect("Lock still has multiple owners");
    final_lock.into_inner().expect("Mutex is poisoned")
}
