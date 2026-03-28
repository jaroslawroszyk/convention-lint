//! Naming convention definitions and stem validation.
//!
//! This module is **entirely agnostic** of any particular build system or
//! configuration format.  It knows nothing about Cargo, CLI arguments, or I/O.
//! Higher-level crates or modules wire these primitives into a concrete
//! linting pipeline.

use std::fmt;
use std::str::FromStr;

use thiserror::Error;

/// Error returned when an unrecognised string is parsed as a [`Convention`].
///
/// This is the [`Err`] type for `<Convention as FromStr>`.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error(
    "unknown convention `{0}`; valid values: \
     `snake_case`, `CamelCase`, `camelCase`, `SCREAMING_SNAKE_CASE`, `kebab-case`"
)]
pub struct UnknownConvention(pub String);

/// A naming convention that file stems must conform to.
///
/// # Parsing
///
/// Conventions are parsed from plain string identifiers.
/// `PascalCase` is accepted as an alias for [`CamelCase`](Self::CamelCase).
///
/// ```
/// use convention_lint::Convention;
///
/// let c: Convention = "snake_case".parse().unwrap();
/// assert_eq!(c, Convention::SnakeCase);
///
/// assert!("UNKNOWN".parse::<Convention>().is_err());
/// ```
///
/// # Validation
///
/// ```
/// use convention_lint::Convention;
///
/// assert!(Convention::SnakeCase.is_valid("my_service"));
/// assert!(!Convention::SnakeCase.is_valid("MyService"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Convention {
    /// `snake_case` — all lowercase words separated by underscores.
    SnakeCase,
    /// `CamelCase` / `PascalCase` — each word starts with an uppercase letter, no separators.
    CamelCase,
    /// `camelCase` — like `CamelCase` but the first word is lowercase.
    LowerCamelCase,
    /// `SCREAMING_SNAKE_CASE` — all uppercase words separated by underscores.
    ScreamingSnakeCase,
    /// `kebab-case` — all lowercase words separated by hyphens.
    KebabCase,
}

impl Convention {
    /// Returns the canonical string identifier for this convention.
    ///
    /// This is the left-inverse of [`FromStr`]: `s.parse::<Convention>()?.as_str() == s`
    /// holds for all valid identifiers (except the `PascalCase` alias, which maps to
    /// `"CamelCase"`).
    #[inline]
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

    /// Returns `true` if `stem` — a filename **without** its extension — conforms
    /// to this convention.
    ///
    /// # Examples
    ///
    /// ```
    /// use convention_lint::Convention;
    ///
    /// // snake_case
    /// assert!(Convention::SnakeCase.is_valid("hello_world"));
    /// assert!(Convention::SnakeCase.is_valid("foo123"));
    /// assert!(!Convention::SnakeCase.is_valid("Hello_World"));
    /// assert!(!Convention::SnakeCase.is_valid("hello__world")); // double underscore
    /// assert!(!Convention::SnakeCase.is_valid("hello_"));       // trailing underscore
    ///
    /// // CamelCase
    /// assert!(Convention::CamelCase.is_valid("MyService"));
    /// assert!(!Convention::CamelCase.is_valid("my_service"));
    ///
    /// // SCREAMING_SNAKE_CASE
    /// assert!(Convention::ScreamingSnakeCase.is_valid("MY_CONST"));
    /// assert!(!Convention::ScreamingSnakeCase.is_valid("my_const"));
    ///
    /// // kebab-case
    /// assert!(Convention::KebabCase.is_valid("my-service"));
    /// assert!(!Convention::KebabCase.is_valid("my--service")); // double hyphen
    /// ```
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
