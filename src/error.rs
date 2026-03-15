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

    /// The manifest file contains invalid TOML.
    #[error("cannot parse `{path}`: {source}")]
    Toml {
        /// Path to the manifest that could not be parsed.
        path: PathBuf,
        /// Underlying TOML parse error.
        #[source]
        source: toml::de::Error,
    },

    /// The `[package.metadata.convention-lint]` section is absent from the manifest.
    #[error("`[package.metadata.convention-lint]` section not found in `{0}`")]
    MissingSection(PathBuf),

    /// The metadata section exists but is not a TOML table.
    #[error("`[package.metadata.convention-lint]` must be a TOML table")]
    InvalidSection,

    /// The `dirs` sub-table exists but is not a TOML table.
    #[error("`[package.metadata.convention-lint.dirs]` must be a TOML table")]
    InvalidDirsTable,

    /// A convention entry value is not a plain string.
    #[error("value for key `{key}` must be a plain string (e.g. `\"snake_case\"`)")]
    InvalidConventionValue {
        /// The TOML key whose value was not a string.
        key: String,
    },

    /// The convention string is not one of the recognised identifiers.
    #[error(
        "unknown convention `{value}` for extension `{ext}`; \
         valid values: `snake_case`, `CamelCase`, `camelCase`, \
         `SCREAMING_SNAKE_CASE`, `kebab-case`"
    )]
    UnknownConvention {
        /// The file extension the convention was configured for.
        ext: String,
        /// The unrecognised convention string supplied by the user.
        value: String,
    },
}
