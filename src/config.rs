//! Configuration loading from `Cargo.toml` metadata.
//!
//! The configuration lives inside the consuming project's `Cargo.toml` under
//! `[package.metadata.convention-lint]`.  That table maps file extensions to
//! convention identifiers, with an optional `dirs` sub-table that restricts
//! which directories are searched for each extension.
//!
//! ```toml
//! [package.metadata.convention-lint]
//! idl = "snake_case"
//! rs  = "CamelCase"
//!
//! [package.metadata.convention-lint.dirs]
//! idl = ["src/idl"]
//! ```

use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr as _;

use crate::convention::Convention;
use crate::error::Error;

#[derive(Debug, Deserialize)]
struct CargoManifest {
    package: Option<PackageSection>,
    workspace: Option<WorkspaceSection>,
}

#[derive(Debug, Deserialize)]
struct PackageSection {
    metadata: Option<MetadataSection>,
}

#[derive(Debug, Deserialize)]
struct WorkspaceSection {
    metadata: Option<MetadataSection>,
}

#[derive(Debug, Deserialize)]
struct MetadataSection {
    #[serde(rename = "convention-lint")]
    convention_lint: Option<ConventionLintTable>,
}

#[derive(Debug, Deserialize)]
struct ConventionLintTable {
    /// Przechwytuje wszystko co nie jest zarezerwowane (np. idl, rs)
    #[serde(flatten)]
    rules: HashMap<String, String>,

    /// Opcjonalna tabela z listą katalogów
    dirs: Option<HashMap<String, Vec<PathBuf>>>,
}

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

/// Linter configuration loaded from `[package.metadata.convention-lint]`.
///
/// Build this with [`load_config`], then pass it to [`crate::lint::run`].
///
/// # Example
///
/// ```no_run
/// use convention_lint::config::load_config;
///
/// let cfg = load_config(std::path::Path::new("Cargo.toml")).unwrap();
/// println!("{} rule(s) loaded", cfg.rules.len());
/// ```

#[derive(Debug, Default)]
pub struct Config {
    /// Maps a file extension (without leading `.`) to the required naming
    /// convention.
    pub rules: HashMap<String, Convention>,
    /// Maps a file extension to the list of directories that should be
    /// scanned.  Paths are relative to the project root.  When an extension
    /// has no entry here the entire project root is scanned recursively.
    pub dirs: HashMap<String, Vec<PathBuf>>,
}

// ---------------------------------------------------------------------------
// Loader
// ---------------------------------------------------------------------------

/// Parse a [`Config`] from the `Cargo.toml` manifest at `manifest_path`.
///
/// The function reads and parses the file, then extracts the
/// `[package.metadata.convention-lint]` table.  Every key in that table
/// (except the reserved `dirs` sub-table) is interpreted as a file extension
/// mapped to a convention string.
///
/// # Errors
///
/// | Situation | [`Error`] variant |
/// |---|---|
/// | File unreadable | [`Error::Io`] |
/// | Not valid TOML | [`Error::Toml`] |
/// | Section absent | [`Error::MissingSection`] |
/// | Section not a table | [`Error::InvalidSection`] |
/// | `dirs` not a table | [`Error::InvalidDirsTable`] |
/// | Convention value not a string | [`Error::InvalidConventionValue`] |
/// | Unrecognised convention string | [`Error::UnknownConvention`] |
pub fn load_config(manifest_path: &Path) -> Result<Config, Error> {
    let content = std::fs::read_to_string(manifest_path).map_err(|source| Error::Io {
        path: manifest_path.to_owned(),
        source,
    })?;

    let manifest: CargoManifest = toml::from_str(&content).map_err(|source| Error::Toml {
        path: manifest_path.to_owned(),
        source,
    })?;

    let raw_lint = manifest
        .package
        .and_then(|p| p.metadata)
        .and_then(|m| m.convention_lint)
        .or_else(|| {
            manifest
                .workspace
                .and_then(|w| w.metadata)
                .and_then(|m| m.convention_lint)
        })
        .ok_or_else(|| Error::MissingSection(manifest_path.to_owned()))?;

    let mut rules = HashMap::new();

    for (ext, raw_conv) in raw_lint.rules {
        if ext == "dirs" {
            continue;
        }

        let conv = Convention::from_str(&raw_conv).map_err(|_| Error::UnknownConvention {
            ext: ext.clone(),
            value: raw_conv,
        })?;
        rules.insert(ext, conv);
    }

    Ok(Config {
        rules,
        dirs: raw_lint.dirs.unwrap_or_default(),
    })
}
