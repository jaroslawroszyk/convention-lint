//! # convention-lint
//!
//! A file-naming convention linter for Rust projects, configurable via `Cargo.toml` metadata.
//!
//! It enforces naming conventions (like `snake_case` or `CamelCase`) for any files in your
//! project, not just Rust source files. It uses parallel directory traversal (same as ripgrep)
//! and respects `.gitignore`.
//!
//! ## Configuration
//!
//! Add `[[package.metadata.convention-lint.checks]]` or `[[workspace.metadata.convention-lint.checks]]`
//! entries to your `Cargo.toml`.
//!
//! ### Example
//!
//! ```toml
//! [[workspace.metadata.convention-lint.checks]]
//! dirs      = ["src/idl", "proto"]
//! include   = ["*.idl", "*.proto"]
//! exclude   = ["**/legacy_*.proto"]
//! format    = "snake_case"
//! recursive = true
//! ```
//!
//! ### Configuration Fields
//!
//! | Field | Required | Default | Description |
//! |-------|----------|---------|-------------|
//! | `dirs` | **Yes** | — | Directories to scan. Supports globs like `src/*/tests`. **Cannot be empty.** |
//! | `include` | No | `["*"]` | Glob patterns for files to check (e.g. `["*.rs"]`). |
//! | `exclude` | No | `[]` | Glob patterns for files to skip (takes priority over `include`). |
//! | `format` | **Yes** | — | Convention to enforce: `snake_case`, `CamelCase`, `camelCase`, `SCREAMING_SNAKE_CASE`, `kebab-case`. |
//! | `recursive` | No | `true` | If `false`, the linter won't enter subdirectories. |
//!
//! ## Supported Conventions
//!
//! | Identifier | Example |
//! |------------|---------|
//! | `snake_case` | `my_service` |
//! | `CamelCase` | `MyService` (Alias: `PascalCase`) |
//! | `camelCase` | `myService` |
//! | `SCREAMING_SNAKE_CASE` | `MY_SERVICE` |
//! | `kebab-case` | `my-service` |
//!
//! ## Usage
//!
//! Install the tool:
//! ```sh
//! cargo install convention-lint
//! ```
//!
//! Run it in your project root:
//! ```sh
//! cargo convention-lint
//! ```
//!
//! ## Library Usage
//!
//! ```no_run
//! use convention_lint::{config::load_config, lint::run};
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let manifest_path = Path::new("Cargo.toml");
//! let project_root = Path::new(".");
//!
//! let cfg = load_config(manifest_path)?;
//! let violations = run(&cfg, project_root);
//!
//! for v in &violations {
//!     eprintln!("{v}");
//! }
//!
//! if !violations.is_empty() {
//!     std::process::exit(1);
//! }
//! # Ok(())
//! # }
//! ```

pub mod config;
pub mod convention;
pub mod core;
pub mod error;
pub mod lint;

pub use convention::Convention;
pub use error::Error;
pub use lint::Violation;
