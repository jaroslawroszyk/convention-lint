//! Configuration loading from `Cargo.toml` metadata.
//!
//! Checks are defined as a list of `[[...checks]]` entries under
//! `[package.metadata.convention-lint]` or `[workspace.metadata.convention-lint]`.
//!
//! Each entry specifies which directories to scan, which files to include/exclude,
//! and the naming convention to enforce.
//!
//! # Example
//!
//! ```toml
//! [[package.metadata.convention-lint.checks]]
//! dirs    = ["src/idl", "proto"]
//! include = ["*.idl", "*.proto"]
//! exclude = ["legacy_*.proto"]
//! format  = "snake_case"
//! ```

use crate::core::{Convention, Matcher};
use crate::error::Error;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Debug, Deserialize)]
struct CargoManifest {
    package: Option<MetadataWrapper>,
    workspace: Option<MetadataWrapper>,
}

#[derive(Debug, Deserialize)]
struct MetadataWrapper {
    metadata: Option<MetadataSection>,
}

#[derive(Debug, Deserialize)]
struct MetadataSection {
    #[serde(rename = "convention-lint")]
    convention_lint: Option<ConventionLintTable>,
}

#[derive(Debug, Deserialize)]
struct ConventionLintTable {
    checks: Vec<RawCheck>,
}

const fn default_recursive() -> bool {
    true
}

#[derive(Debug, Deserialize)]
struct RawCheck {
    dirs: Vec<PathBuf>,
    #[serde(default)]
    include: Vec<String>,
    #[serde(default)]
    exclude: Vec<String>,
    format: String,
    #[serde(default = "default_recursive")]
    recursive: bool,
}

/// Linter configuration loaded from a project manifest.
#[derive(Debug)]
pub struct Config {
    /// A list of resolved rules to be applied during the linting process.
    pub rules: Vec<ResolvedRule>,
}

/// A single, fully-resolved linting rule.
#[derive(Debug)]
pub struct ResolvedRule {
    /// Directories to be scanned for this rule.
    pub dirs: Vec<PathBuf>,
    /// The glob matcher used to include or exclude specific files.
    pub matcher: Matcher,
    /// The naming convention to be enforced for matched files.
    pub convention: Convention,
    /// Whether to scan directories recursively (currently always true, planned for future support of non-recursive rules)
    pub recursive: bool, // TODO: issue #2 - Add support for non-recursive rules in a future issue (e.g. via `recursive = false` flag in the config
}

/// Loads the linter configuration from the specified `Cargo.toml` manifest.
///
/// This function looks for configuration in `[package.metadata.convention-lint]`
/// and falls back to `[workspace.metadata.convention-lint]` if the package-specific
/// section is missing.
///
/// # Errors
///
/// Returns an error if the file cannot be read, contains invalid TOML,
/// or is missing the required metadata sections.
pub fn load_config(manifest_path: &Path) -> Result<Config, Error> {
    let content = std::fs::read_to_string(manifest_path).map_err(|source| Error::Io {
        path: manifest_path.to_owned(),
        source,
    })?;

    let manifest: CargoManifest = toml::from_str(&content).map_err(|source| Error::Toml {
        path: manifest_path.to_owned(),
        source,
    })?;

    let mut all_raw_checks = Vec::new();

    if let Some(checks) = manifest
        .package
        .and_then(|p| p.metadata)
        .and_then(|m| m.convention_lint)
        .map(|cl| cl.checks)
    {
        all_raw_checks.extend(checks);
    }

    if let Some(checks) = manifest
        .workspace
        .and_then(|w| w.metadata)
        .and_then(|m| m.convention_lint)
        .map(|cl| cl.checks)
    {
        all_raw_checks.extend(checks);
    }

    if all_raw_checks.is_empty() {
        return Err(Error::MissingSection(manifest_path.to_owned()));
    }

    let mut rules = Vec::new();
    for raw in all_raw_checks {
        let error_context = if raw.include.is_empty() {
            "all files".to_string()
        } else {
            raw.include.join(", ")
        };

        let convention =
            Convention::from_str(&raw.format).map_err(|_| Error::UnknownConvention {
                context: error_context,
                value: raw.format.clone(),
            })?;

        let matcher =
            Matcher::new(&raw.include, &raw.exclude).map_err(|_| Error::InvalidSection)?;

        rules.push(ResolvedRule {
            dirs: raw.dirs,
            matcher,
            convention,
            recursive: raw.recursive, // TODO: issue #2 - Add support for non-recursive rules in a future issue (e.g. via `recursive = false` flag in the config
        });
    }

    Ok(Config { rules })
}
