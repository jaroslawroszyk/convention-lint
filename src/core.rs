//! Naming convention definitions and stem validation.
//!
//! This module is **entirely agnostic** of any particular build system or
//! configuration format. It knows nothing about Cargo, CLI arguments, or I/O.
//! Higher-level crates or modules wire these primitives into a concrete
//! linting pipeline.

use globset::{Glob, GlobSet, GlobSetBuilder};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

/// Error returned when an unrecognized string is parsed as a [`Convention`].
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error(
    "unknown convention `{0}`; valid values: `snake_case`, `CamelCase`, `camelCase`, `SCREAMING_SNAKE_CASE`, `kebab-case`"
)]
pub struct UnknownConvention(pub String);

/// Supported naming conventions for file stems.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Convention {
    /// `snake_case` (e.g. `my_service`)
    SnakeCase,
    /// `CamelCase` or `PascalCase` (e.g. `MyService`)
    CamelCase,
    /// `camelCase` (e.g. `myService`)
    LowerCamelCase,
    /// `SCREAMING_SNAKE_CASE` (e.g. `MY_SERVICE`)
    ScreamingSnakeCase,
    /// `kebab-case` (e.g. `my-service`)
    KebabCase,
}

impl Convention {
    /// Returns the canonical string identifier for this convention.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::SnakeCase => "snake_case",
            Self::CamelCase => "CamelCase",
            Self::LowerCamelCase => "camelCase",
            Self::ScreamingSnakeCase => "SCREAMING_SNAKE_CASE",
            Self::KebabCase => "kebab-case",
        }
    }

    /// Checks if the given file stem conforms to this convention.
    ///
    /// An empty stem is considered invalid for all conventions.
    /// The stem is expected to be the file name without extension or directory components.
    /// For example, for `src/my_service.rs`, the stem would be `my_service`.
    ///
    /// This method does not perform any filesystem operations and assumes the input
    /// is a valid file stem. It validates the string based on the following rules:
    ///
    /// - **`snake_case`**: Lowercase letters, digits, and underscores. Must start with
    ///   a lowercase letter, cannot end with an underscore, and no consecutive underscores (`__`).
    /// - **`CamelCase`**: Alphanumeric characters. Must start with an uppercase letter.
    ///   No separators allowed.
    /// - **`camelCase`**: Alphanumeric characters. Must start with a lowercase letter.
    ///   No separators allowed.
    /// - **`SCREAMING_SNAKE_CASE`**: Uppercase letters, digits, and underscores. Must start
    ///   with an uppercase letter, cannot end with an underscore, and no consecutive underscores.
    /// - **`kebab-case`**: Lowercase letters, digits, and hyphens. Must start with
    ///   a lowercase letter, cannot end with a hyphen, and no consecutive hyphens (`--`).
    #[must_use]
    pub fn is_valid(&self, stem: &str) -> bool {
        let Some(first) = stem.chars().next() else {
            return false;
        };
        match self {
            Self::SnakeCase => {
                first.is_ascii_lowercase()
                    && stem
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
                    && !stem.contains("__")
                    && !stem.ends_with('_')
            }
            Self::CamelCase => {
                first.is_ascii_uppercase() && stem.chars().all(|c| c.is_ascii_alphanumeric())
            }
            Self::LowerCamelCase => {
                first.is_ascii_lowercase() && stem.chars().all(|c| c.is_ascii_alphanumeric())
            }
            Self::ScreamingSnakeCase => {
                first.is_ascii_uppercase()
                    && stem
                        .chars()
                        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
                    && !stem.contains("__")
                    && !stem.ends_with('_')
            }
            Self::KebabCase => {
                first.is_ascii_lowercase()
                    && stem
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
                    && !stem.contains("--")
                    && !stem.ends_with('-')
            }
        }
    }
}

impl FromStr for Convention {
    type Err = UnknownConvention;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "snake_case" => Ok(Self::SnakeCase),
            "CamelCase" | "PascalCase" => Ok(Self::CamelCase),
            "camelCase" => Ok(Self::LowerCamelCase),
            "SCREAMING_SNAKE_CASE" => Ok(Self::ScreamingSnakeCase),
            "kebab-case" => Ok(Self::KebabCase),
            other => Err(UnknownConvention(other.to_owned())),
        }
    }
}

impl fmt::Display for Convention {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A file naming violation detected by the linter.
#[derive(Debug, Clone)]
pub struct Matcher {
    include: Option<GlobSet>,
    exclude: Option<GlobSet>,
}

impl Matcher {
    /// Creates a new [`Matcher`] instance.
    ///
    /// # Parameters
    /// * `include` - list of glob patterns for files to be checked.
    /// * `exclude` - list of glob patterns for files to be ignored.
    ///
    /// # Errors
    /// Returns an error if any of the glob patterns are invalid.
    pub fn new(include: &[String], exclude: &[String]) -> Result<Self, globset::Error> {
        let build_set = |patterns: &[String]| -> Result<Option<GlobSet>, globset::Error> {
            if patterns.is_empty() {
                return Ok(None);
            }
            let mut builder = GlobSetBuilder::new();
            for p in patterns {
                builder.add(Glob::new(p)?);
            }
            Ok(Some(builder.build()?))
        };

        Ok(Self {
            include: build_set(include)?,
            exclude: build_set(exclude)?,
        })
    }

    /// Checks if the given filename matches the include patterns and does not match the exclude patterns.
    #[must_use]
    pub fn is_match(&self, filename: &str) -> bool {
        if let Some(ref exc) = self.exclude {
            if exc.is_match(filename) {
                return false;
            }
        }

        self.include
            .as_ref()
            .is_none_or(|inc| inc.is_match(filename))
    }
}
