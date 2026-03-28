//! Integration tests for [`config::load_config`] and [`lint::run`].
//!
//! Each test builds a real filesystem layout inside a [`tempfile::TempDir`]
//! so the linter runs against actual files rather than mocked data.

use std::fs;
use std::path::Path;

use tempfile::TempDir;

use convention_lint::Convention;
use convention_lint::config::load_config;
use convention_lint::error::Error;
use convention_lint::lint::run;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a directory tree and write files.
///
/// `paths` is a list of relative paths; directories are created automatically.
fn scaffold(root: &Path, paths: &[&str]) {
    for rel in paths {
        let abs = root.join(rel);
        if let Some(parent) = abs.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&abs, b"").unwrap();
    }
}

/// Write a minimal `Cargo.toml`-style manifest to `root/Cargo.toml` with
/// the `[package.metadata.convention-lint]` section provided as raw TOML.
fn write_manifest(root: &Path, metadata_section: &str) -> std::path::PathBuf {
    let content = format!(
        "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n{metadata_section}"
    );
    let path = root.join("Cargo.toml");
    fs::write(&path, content).unwrap();
    path
}

// ---------------------------------------------------------------------------
// load_config — error paths
// ---------------------------------------------------------------------------

#[test]
fn load_config_missing_file_returns_io_error() {
    let result = load_config(Path::new("/nonexistent/Cargo.toml"));
    assert!(matches!(result, Err(Error::Io { .. })));
}

#[test]
fn load_config_invalid_toml_returns_parse_error() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("Cargo.toml"), b"[not valid toml {{{{").unwrap();
    let result = load_config(&dir.path().join("Cargo.toml"));
    assert!(matches!(result, Err(Error::Toml { .. })));
}

#[test]
fn load_config_missing_section_returns_error() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        b"[package]\nname=\"x\"\nversion=\"0.1.0\"\nedition=\"2021\"\n",
    )
    .unwrap();
    let result = load_config(&dir.path().join("Cargo.toml"));
    assert!(matches!(result, Err(Error::MissingSection(_))));
}

#[test]
fn load_config_unknown_convention_returns_error() {
    let dir = TempDir::new().unwrap();
    let manifest = write_manifest(
        dir.path(),
        "[package.metadata.convention-lint]\nidl = \"WrongCase\"\n",
    );
    let result = load_config(&manifest);
    assert!(
        matches!(result, Err(Error::UnknownConvention { .. })),
        "expected UnknownConvention, got {result:?}"
    );
}

#[test]
fn load_config_non_string_value_returns_error() {
    let dir = TempDir::new().unwrap();
    let manifest = write_manifest(dir.path(), "[package.metadata.convention-lint]\nidl = 42\n");
    let result = load_config(&manifest);
    assert!(matches!(result, Err(Error::InvalidConventionValue { .. })));
}

// ---------------------------------------------------------------------------
// load_config — happy paths
// ---------------------------------------------------------------------------

#[test]
fn load_config_parses_rules_and_dirs() {
    let dir = TempDir::new().unwrap();
    let manifest = write_manifest(
        dir.path(),
        r#"
[package.metadata.convention-lint]
idl = "snake_case"
rs  = "CamelCase"

[package.metadata.convention-lint.dirs]
idl = ["src/idl", "proto"]
"#,
    );

    let cfg = load_config(&manifest).unwrap();

    assert_eq!(cfg.rules.get("idl"), Some(&Convention::SnakeCase));
    assert_eq!(cfg.rules.get("rs"), Some(&Convention::CamelCase));
    assert_eq!(cfg.rules.get("proto"), None);

    let idl_dirs = cfg.dirs.get("idl").unwrap();
    assert_eq!(idl_dirs.len(), 2);
    assert!(idl_dirs.iter().any(|p| p.ends_with("src/idl")));
    assert!(idl_dirs.iter().any(|p| p.ends_with("proto")));
}

// ---------------------------------------------------------------------------
// run — no violations
// ---------------------------------------------------------------------------

#[test]
fn run_all_valid_produces_no_violations() {
    let dir = TempDir::new().unwrap();
    scaffold(
        dir.path(),
        &["src/idl/my_service.idl", "src/idl/another_one.idl"],
    );
    let manifest = write_manifest(
        dir.path(),
        r#"
[package.metadata.convention-lint]
idl = "snake_case"

[package.metadata.convention-lint.dirs]
idl = ["src/idl"]
"#,
    );

    let cfg = load_config(&manifest).unwrap();
    let violations = run(&cfg, dir.path());

    assert!(
        violations.is_empty(),
        "expected no violations, got: {violations:#?}"
    );
}

// ---------------------------------------------------------------------------
// run — violations detected
// ---------------------------------------------------------------------------

#[test]
fn run_detects_bad_stems() {
    let dir = TempDir::new().unwrap();
    scaffold(
        dir.path(),
        &[
            "idl/good_name.idl",
            "idl/BadName.idl",
            "idl/also_good.idl",
            "idl/alsoBAD.idl",
        ],
    );
    let manifest = write_manifest(
        dir.path(),
        r#"
[package.metadata.convention-lint]
idl = "snake_case"

[package.metadata.convention-lint.dirs]
idl = ["idl"]
"#,
    );

    let cfg = load_config(&manifest).unwrap();
    let violations = run(&cfg, dir.path());

    let bad_stems: Vec<&str> = violations.iter().map(|v| v.stem.as_str()).collect();
    assert!(
        bad_stems.contains(&"BadName"),
        "missing BadName in {bad_stems:?}"
    );
    assert!(
        bad_stems.contains(&"alsoBAD"),
        "missing alsoBAD in {bad_stems:?}"
    );
    assert!(
        !bad_stems.contains(&"good_name"),
        "good_name should not be a violation"
    );
    assert!(
        !bad_stems.contains(&"also_good"),
        "also_good should not be a violation"
    );
    assert_eq!(violations.len(), 2);
}

#[test]
fn run_violation_paths_are_relative_to_project_root() {
    let dir = TempDir::new().unwrap();
    scaffold(dir.path(), &["src/idl/BadName.idl"]);
    let manifest = write_manifest(
        dir.path(),
        r#"
[package.metadata.convention-lint]
idl = "snake_case"

[package.metadata.convention-lint.dirs]
idl = ["src/idl"]
"#,
    );

    let cfg = load_config(&manifest).unwrap();
    let violations = run(&cfg, dir.path());

    assert_eq!(violations.len(), 1);
    // path must be relative, not absolute
    assert!(violations[0].path.is_relative());
    assert_eq!(violations[0].path, Path::new("src/idl/BadName.idl"));
}

#[test]
fn run_violation_carries_expected_convention() {
    let dir = TempDir::new().unwrap();
    scaffold(dir.path(), &["my_Service.idl"]);
    let manifest = write_manifest(
        dir.path(),
        "[package.metadata.convention-lint]\nidl = \"snake_case\"\n",
    );

    let cfg = load_config(&manifest).unwrap();
    let violations = run(&cfg, dir.path());

    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].stem, "my_Service");
    assert_eq!(violations[0].expected, Convention::SnakeCase);
}

// ---------------------------------------------------------------------------
// run — extension filtering
// ---------------------------------------------------------------------------

#[test]
fn run_ignores_files_with_different_extension() {
    let dir = TempDir::new().unwrap();
    // Only rule is for `.idl`; the `.rs` file has a bad stem but must be ignored
    scaffold(dir.path(), &["BadName.rs", "GoodEnough.idl"]);
    let manifest = write_manifest(
        dir.path(),
        "[package.metadata.convention-lint]\nidl = \"CamelCase\"\n",
    );

    let cfg = load_config(&manifest).unwrap();
    let violations = run(&cfg, dir.path());

    assert!(
        violations.is_empty(),
        "`.rs` file should not have been checked, got: {violations:#?}"
    );
}

// ---------------------------------------------------------------------------
// run — directory scoping
// ---------------------------------------------------------------------------

#[test]
fn run_only_scans_configured_dirs() {
    let dir = TempDir::new().unwrap();
    scaffold(
        dir.path(),
        &[
            "src/idl/good_name.idl",
            "other/BadName.idl", // outside configured dir — must be ignored
        ],
    );
    let manifest = write_manifest(
        dir.path(),
        r#"
[package.metadata.convention-lint]
idl = "snake_case"

[package.metadata.convention-lint.dirs]
idl = ["src/idl"]
"#,
    );

    let cfg = load_config(&manifest).unwrap();
    let violations = run(&cfg, dir.path());

    assert!(
        violations.is_empty(),
        "file outside configured dir should be ignored, got: {violations:#?}"
    );
}

#[test]
fn run_without_dirs_config_scans_whole_project() {
    let dir = TempDir::new().unwrap();
    scaffold(
        dir.path(),
        &[
            "src/a/good_name.idl",
            "src/b/another_good.idl",
            "BadName.idl",
        ],
    );
    let manifest = write_manifest(
        dir.path(),
        "[package.metadata.convention-lint]\nidl = \"snake_case\"\n",
    );

    let cfg = load_config(&manifest).unwrap();
    let violations = run(&cfg, dir.path());

    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].stem, "BadName");
}

// ---------------------------------------------------------------------------
// run — hidden dirs and `target/` are skipped
// ---------------------------------------------------------------------------

#[test]
fn run_skips_hidden_directories() {
    let dir = TempDir::new().unwrap();
    scaffold(
        dir.path(),
        &[".hidden/BadName.idl", "visible/good_name.idl"],
    );
    let manifest = write_manifest(
        dir.path(),
        "[package.metadata.convention-lint]\nidl = \"snake_case\"\n",
    );

    let cfg = load_config(&manifest).unwrap();
    let violations = run(&cfg, dir.path());

    assert!(
        violations.is_empty(),
        "files in hidden dirs should be skipped, got: {violations:#?}"
    );
}

#[test]
fn run_skips_target_directory() {
    let dir = TempDir::new().unwrap();
    scaffold(dir.path(), &["target/BadName.idl", "src/good_name.idl"]);
    let manifest = write_manifest(
        dir.path(),
        "[package.metadata.convention-lint]\nidl = \"snake_case\"\n",
    );

    let cfg = load_config(&manifest).unwrap();
    let violations = run(&cfg, dir.path());

    assert!(
        violations.is_empty(),
        "`target/` should be skipped, got: {violations:#?}"
    );
}

// ---------------------------------------------------------------------------
// Violation::Display
// ---------------------------------------------------------------------------

#[test]
fn violation_display_format() {
    use convention_lint::lint::Violation;
    use std::path::PathBuf;

    let v = Violation {
        path: PathBuf::from("src/idl/BadName.idl"),
        stem: "BadName".to_owned(),
        expected: Convention::SnakeCase,
    };
    let s = v.to_string();
    assert!(
        s.starts_with("error[convention]:"),
        "unexpected prefix: {s}"
    );
    assert!(s.contains("BadName"), "stem missing from output: {s}");
    assert!(
        s.contains("snake_case"),
        "convention missing from output: {s}"
    );
}
