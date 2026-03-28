//! # convention-lint
//!
//! A file-naming convention linter configurable via `Cargo.toml` metadata.
//!
//! Drop a `[package.metadata.convention-lint]` section into the manifest of
//! the project you want to lint:
//!
//! ```toml
//! [package.metadata.convention-lint]
//! idl = "snake_case"
//! rs  = "CamelCase"
//!
//! [package.metadata.convention-lint.dirs]
//! idl = ["src/idl"]   # optional; omit to scan the whole project
//! ```
//!
//! Then invoke the linter as a Cargo subcommand:
//!
//! ```sh
//! cargo convention-lint
//! cargo convention-lint --manifest-path path/to/Cargo.toml
//! ```
//!
//! ## Library usage
//!
//! The crate exposes its full API so it can be embedded in build-scripts or
//! other tooling:
//!
//! ```no_run
//! use convention_lint::{config::load_config, lint::run};
//!
//! let cfg = load_config(std::path::Path::new("Cargo.toml")).unwrap();
//! let violations = run(&cfg, std::path::Path::new("."));
//! for v in &violations {
//!     eprintln!("{v}");
//! }
//! std::process::exit(if violations.is_empty() { 0 } else { 1 });
//! ```

pub mod config;
pub mod convention;
pub mod error;
pub mod lint;

// Convenience re-exports at the crate root.
pub use convention::Convention;
pub use error::Error;
pub use lint::Violation;
