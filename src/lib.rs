//! # convention-lint
//!
//! A file-naming convention linter for Rust projects, configurable via `Cargo.toml` metadata.
//!
//! It enforces naming conventions (like `snake_case` or `CamelCase`) for any files in your
//! project, not just Rust source files.
//!
//! ## Configuration
//!
//! Add a `[[package.metadata.convention-lint.checks]]` section to your `Cargo.toml`:
//!
//! ```toml
//! [[package.metadata.convention-lint.checks]]
//! dirs    = ["src/idl", "proto"]
//! include = ["*.idl", "*.proto"]
//! format  = "snake_case"
//!
//! [[package.metadata.convention-lint.checks]]
//! dirs    = ["src"]
//! include = ["*.rs"]
//! format  = "snake_case"
//! ```
//!
//! ## Supported conventions
//!
//! | Identifier            | Example       |
//! |-----------------------|---------------|
//! | `snake_case`          | `my_service`  |
//! | `CamelCase`           | `MyService`   |
//! | `PascalCase`          | `MyService`   |
//! | `camelCase`           | `myService`   |
//! | `SCREAMING_SNAKE_CASE`| `MY_SERVICE`  |
//! | `kebab-case`          | `my-service`  |
//!
//! ## Usage
//!
//! Invoke the linter as a Cargo subcommand:
//!
//! ```sh
//! cargo convention-lint
//! ```
//!
//! ## Library usage
//!
//! The crate exposes its full API so it can be embedded in build-scripts or other tooling:
//!
//! ```no_run
//! use convention_lint::{config::load_config, lint::run};
//! use std::path::Path;
//!
//! let manifest_path = Path::new("Cargo.toml");
//! let project_root = Path::new(".");
//!
//! let cfg = load_config(manifest_path).expect("Failed to load config");
//! let violations = run(&cfg, project_root);
//!
//! for v in &violations {
//!     eprintln!("{v}");
//! }
//!
//! if !violations.is_empty() {
//!     std::process::exit(1);
//! }
//! ```

pub mod config;
pub mod convention;
pub mod core;
pub mod error;
pub mod lint;

pub use convention::Convention;
pub use error::Error;
pub use lint::Violation;
