//! Core linting logic — filesystem walk and violation collection.

use ignore::WalkBuilder;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

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
/// # Panics
///
/// This function will panic if the internal mutex is poisoned, which occurs
/// if a thread panics while holding the lock.
#[must_use]
pub fn run(config: &Config, project_root: &Path) -> Vec<Violation> {
    let violations = Arc::new(Mutex::new(Vec::new()));

    for (ext, convention) in &config.rules {
        let targets = search_dirs(config, ext, project_root);

        for target in targets {
            let violations_lock = Arc::clone(&violations);
            let ext_owned = ext.clone();
            let conv_owned = convention.clone();
            let root_owned = project_root.to_path_buf();

            if !target.exists() {
                eprintln!(
                    "warning: directory `{}` for extension `.{ext_owned}` does not exist",
                    target.display()
                );
                continue;
            }

            let walker = WalkBuilder::new(target)
                .standard_filters(true)
                .hidden(false)
                .parents(false)
                .build_parallel();

            walker.run(|| {
                let v_inner = Arc::clone(&violations_lock);
                let e_inner = ext_owned.clone();
                let c_inner = conv_owned.clone();
                let r_inner = root_owned.clone();

                Box::new(move |result| {
                    if let Ok(entry) = result {
                        let path = entry.path();
                        let file_name = entry.file_name().to_string_lossy();

                        // Collapse hidden/target checks and use is_some_and
                        if (file_name == "target"
                            || (file_name.starts_with('.') && entry.depth() > 0))
                            && entry.file_type().is_some_and(|ft| ft.is_dir())
                        {
                            return ignore::WalkState::Skip;
                        }

                        // Use is_some_and and combine conditions for files
                        if entry.file_type().is_some_and(|f| f.is_file())
                            && path.extension().and_then(|s| s.to_str()) == Some(&e_inner)
                        {
                            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                                if !c_inner.is_valid(stem) {
                                    let rel_path =
                                        path.strip_prefix(&r_inner).unwrap_or(path).to_path_buf();

                                    v_inner.lock().expect("mutex poisoned").push(Violation {
                                        path: rel_path,
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

    let final_lock = Arc::try_unwrap(violations).expect("Lock still has multiple owners");
    final_lock.into_inner().expect("Mutex is poisoned")
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
