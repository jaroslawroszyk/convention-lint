//! Error types for `convention-lint`.

use std::path::PathBuf;
use thiserror::Error;

/// All errors that `convention-lint` can produce.
///
/// This type is `#[non_exhaustive]` so that new variants can be added in
/// minor versions without breaking downstream `match` expressions.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// The manifest file could not be opened or read.
    #[error("cannot read `{path}`: {source}")]
    Io {
        /// Path to the manifest that could not be read.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// The `dirs` list in a check rule is empty.
    #[error("the `dirs` list in a convention-lint check cannot be empty")]
    EmptyDirs,

    /// The manifest file contains invalid TOML.
    #[error("cannot parse `{path}`: {source}")]
    Toml {
        /// Path to the manifest that could not be parsed.
        path: PathBuf,
        /// Underlying TOML parse error.
        #[source]
        source: toml::de::Error,
    },

    /// The `[workspace.metadata.convention-lint]` or `[package.metadata.convention-lint]` section is absent from the manifest.
    #[error(
        "`[workspace.metadata.convention-lint]` or `[package.metadata.convention-lint]` section not found in `{0}`"
    )]
    MissingSection(PathBuf),

    /// The metadata section exists but is not a TOML table.
    #[error(
        "`[workspace.metadata.convention-lint]` or `[package.metadata.convention-lint]` must be a TOML table"
    )]
    InvalidSection,

    /// The convention string is not one of the recognised identifiers.
    #[error(
        "unknown convention `{value}` for pattern(s) [{context}]; \
         valid values: `snake_case`, `CamelCase`, `camelCase`, \
         `SCREAMING_SNAKE_CASE`, `kebab-case`"
    )]
    UnknownConvention {
        /// The file extensions (e.g. `*.idl`) that the unrecognised convention was configured for.
        context: String,
        /// The unrecognised convention string supplied by the user.
        value: String,
    },
}
