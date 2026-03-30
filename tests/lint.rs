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
fn scaffold(root: &Path, paths: &[&str]) {
    for rel in paths {
        let abs = root.join(rel);
        if let Some(parent) = abs.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&abs, b"").unwrap();
    }
}

/// Write a minimal `Cargo.toml`-style manifest to `root/Cargo.toml` with the given metadata section.
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
        r#"
[[package.metadata.convention-lint.checks]]
dirs = ["src"]
include = ["*.idl"]
format = "WrongCase"
"#,
    );
    let result = load_config(&manifest);
    assert!(
        matches!(result, Err(Error::UnknownConvention { .. })),
        "expected UnknownConvention, got {result:?}"
    );
}

#[test]
fn load_config_non_string_value_returns_toml_error() {
    let dir = TempDir::new().unwrap();
    let manifest = write_manifest(
        dir.path(),
        r#"
[[package.metadata.convention-lint.checks]]
dirs = ["src"]
format = 42
"#,
    );
    let result = load_config(&manifest);
    assert!(matches!(result, Err(Error::Toml { .. })));
}

#[test]
fn load_config_from_workspace_metadata() {
    let dir = TempDir::new().unwrap();
    let content = r#"
[workspace]
members = ["core"]

[[workspace.metadata.convention-lint.checks]]
dirs = ["src"]
include = ["*.rs"]
format = "snake_case"
"#;
    let path = dir.path().join("Cargo.toml");
    fs::write(&path, content).unwrap();

    let cfg = load_config(&path).expect("Should load from workspace metadata");
    assert_eq!(cfg.rules.len(), 1);
    assert_eq!(cfg.rules[0].convention, Convention::SnakeCase);
}

#[test]
fn load_config_merges_package_and_workspace_rules() {
    let dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "combined-test"
version = "0.1.0"

[[package.metadata.convention-lint.checks]]
dirs = ["idl"]
include = ["*.idl"]
format = "snake_case"

[workspace]
[[workspace.metadata.convention-lint.checks]]
dirs = ["src"]
include = ["*.rs"]
format = "CamelCase"
"#;
    let path = dir.path().join("Cargo.toml");
    std::fs::write(&path, content).unwrap();

    let cfg = load_config(&path).expect("Should load combined metadata");

    assert_eq!(
        cfg.rules.len(),
        2,
        "Should have merged 1 package rule and 1 workspace rule"
    );

    let has_snake = cfg
        .rules
        .iter()
        .any(|r| r.convention == Convention::SnakeCase);
    let has_camel = cfg
        .rules
        .iter()
        .any(|r| r.convention == Convention::CamelCase);

    assert!(has_snake, "Missing snake_case rule from package");
    assert!(has_camel, "Missing CamelCase rule from workspace");
}

#[test]
fn load_config_fails_when_no_metadata_anywhere() {
    let dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "empty-test"
version = "0.1.0"
[workspace]
"#;
    let path = dir.path().join("Cargo.toml");
    std::fs::write(&path, content).unwrap();

    let result = load_config(&path);
    assert!(matches!(result, Err(Error::MissingSection(_))));
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
[[package.metadata.convention-lint.checks]]
dirs = ["src/idl", "proto"]
include = ["*.idl"]
format = "snake_case"

[[package.metadata.convention-lint.checks]]
dirs = ["src"]
include = ["*.rs"]
format = "CamelCase"
"#,
    );

    let cfg = load_config(&manifest).unwrap();

    assert_eq!(cfg.rules.len(), 2);
    assert_eq!(cfg.rules[0].convention, Convention::SnakeCase);
    assert_eq!(cfg.rules[0].dirs.len(), 2);
    assert!(cfg.rules[0].dirs.iter().any(|p| p.ends_with("src/idl")));
    assert!(cfg.rules[0].dirs.iter().any(|p| p.ends_with("proto")));

    assert_eq!(cfg.rules[1].convention, Convention::CamelCase);
    assert_eq!(cfg.rules[1].dirs.len(), 1);
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
[[package.metadata.convention-lint.checks]]
dirs = ["src/idl"]
include = ["*.idl"]
format = "snake_case"
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
[[package.metadata.convention-lint.checks]]
dirs = ["idl"]
include = ["*.idl"]
format = "snake_case"
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
[[package.metadata.convention-lint.checks]]
dirs = ["src/idl"]
include = ["*.idl"]
format = "snake_case"
"#,
    );

    let cfg = load_config(&manifest).unwrap();
    let violations = run(&cfg, dir.path());

    assert_eq!(violations.len(), 1);
    assert!(violations[0].path.is_relative());
    assert_eq!(violations[0].path, Path::new("src/idl/BadName.idl"));
}

#[test]
fn run_violation_carries_expected_convention() {
    let dir = TempDir::new().unwrap();
    scaffold(dir.path(), &["src/my_Service.idl"]);
    let manifest = write_manifest(
        dir.path(),
        r#"
[[package.metadata.convention-lint.checks]]
dirs = ["."]
include = ["*.idl"]
format = "snake_case"
"#,
    );

    let cfg = load_config(&manifest).unwrap();
    let violations = run(&cfg, dir.path());

    assert_eq!(violations.len(), 1);
    let violation = violations
        .iter()
        .find(|v| v.stem == "my_Service")
        .expect("Violation not found");
    assert_eq!(violation.stem, "my_Service");
    assert_eq!(violation.expected, Convention::SnakeCase);
}

// ---------------------------------------------------------------------------
// run — include/exclude filtering
// ---------------------------------------------------------------------------

#[test]
fn run_include_only_filters_by_glob() {
    let dir = TempDir::new().unwrap();
    scaffold(
        dir.path(),
        &[
            "some/dir/goodFile.rs",
            "some/dir/BadFile.py", // not in include → ignored
        ],
    );
    let manifest = write_manifest(
        dir.path(),
        r#"
[[package.metadata.convention-lint.checks]]
dirs = ["some/dir"]
include = ["*.rs"]
format = "camelCase"
"#,
    );

    let cfg = load_config(&manifest).unwrap();
    let violations = run(&cfg, dir.path());

    assert!(
        !violations
            .iter()
            .map(|v| v.stem.as_str())
            .any(|x| x == "BadFile"),
        "BadFile.py should not be checked (not in include)"
    );
    assert!(
        violations.is_empty(),
        "goodFile.rs is valid camelCase, expected no violations, got: {violations:#?}"
    );
}

#[test]
fn run_include_and_exclude_filters_correctly() {
    let dir = TempDir::new().unwrap();
    scaffold(
        dir.path(),
        &[
            "other/dir/BadName.py",
            "other/dir/AnotherBad.sh",
            "other/dir/__init__.py", // excluded
            "other/dir/readme.txt",  // not in include
        ],
    );
    let manifest = write_manifest(
        dir.path(),
        r#"
[[package.metadata.convention-lint.checks]]
dirs = ["other/dir"]
include = ["*.py", "*.sh"]
exclude = ["**/__init__.py"]
format = "PascalCase"
"#,
    );

    let cfg = load_config(&manifest).unwrap();
    let violations = run(&cfg, dir.path());

    let stems: Vec<&str> = violations.iter().map(|v| v.stem.as_str()).collect();
    assert!(
        !stems.contains(&"__init__"),
        "__init__.py should be excluded"
    );
    assert!(
        !stems.contains(&"readme"),
        "readme.txt should not be checked (not in include)"
    );
    assert!(
        violations.is_empty(),
        "BadName and AnotherBad are valid PascalCase, got: {violations:#?}"
    );
}

#[test]
fn run_exclude_only_matches_all_except_excluded() {
    let dir = TempDir::new().unwrap();
    scaffold(
        dir.path(),
        &[
            "other/dir3/GoodFile.txt",
            "other/dir3/the-only-exclude.txt", // excluded
            "other/dir3/bad_file.rs",          // no include → matches all
        ],
    );
    let manifest = write_manifest(
        dir.path(),
        r#"
[[package.metadata.convention-lint.checks]]
dirs = ["other/dir3"]
exclude = ["**/the-only-exclude.txt"]
format = "PascalCase"
"#,
    );

    let cfg = load_config(&manifest).unwrap();
    let violations = run(&cfg, dir.path());

    let stems: Vec<&str> = violations.iter().map(|v| v.stem.as_str()).collect();
    assert!(
        !violations
            .iter()
            .any(|v| v.path.to_string_lossy().contains("the-only-exclude.txt")),
        "the-only-exclude.txt should be excluded"
    );
    assert!(
        stems.contains(&"bad_file"),
        "bad_file.rs should be a PascalCase violation, got: {stems:?}"
    );
    assert!(
        !stems.contains(&"GoodFile"),
        "GoodFile.txt is valid PascalCase"
    );
}

#[test]
fn run_exclude_matches_exact_relative_path() {
    let dir = TempDir::new().unwrap();
    scaffold(
        dir.path(),
        &["core/cli/src/foo-case.rs", "other/src/foo-case.rs"],
    );

    let manifest = write_manifest(
        dir.path(),
        r#"
[[package.metadata.convention-lint.checks]]
dirs = ["."]
include = ["*.rs"]
exclude = ["core/cli/src/foo-case.rs"]
format = "snake_case"
"#,
    );

    let cfg = load_config(&manifest).unwrap();
    let violations = run(&cfg, dir.path());

    assert_eq!(
        violations.len(),
        1,
        "Should find exactly 1 violation, got: {violations:#?}"
    );

    assert_eq!(
        violations[0].path,
        Path::new("other/src/foo-case.rs"),
        "The violation should come from the non-excluded path"
    );

    let stems: Vec<&str> = violations.iter().map(|v| v.stem.as_str()).collect();
    assert!(
        !stems.iter().any(|_s| violations[0]
            .path
            .to_string_lossy()
            .contains("core/cli/src")),
        "The file core/cli/src/foo-case.rs should have been excluded"
    );
}

// ---------------------------------------------------------------------------
// run — extension filtering
// ---------------------------------------------------------------------------

#[test]
fn run_ignores_files_with_different_extension() {
    let dir = TempDir::new().unwrap();
    scaffold(dir.path(), &["BadName.rs", "GoodEnough.idl"]);
    let manifest = write_manifest(
        dir.path(),
        r#"
[[package.metadata.convention-lint.checks]]
dirs = ["."]
include = ["*.idl"]
format = "CamelCase"
"#,
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
[[package.metadata.convention-lint.checks]]
dirs = ["src/idl"]
include = ["*.idl"]
format = "snake_case"
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
fn run_with_root_dir_scans_whole_project() {
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
        r#"
[[package.metadata.convention-lint.checks]]
dirs = ["."]
include = ["*.idl"]
format = "snake_case"
"#,
    );

    let cfg = load_config(&manifest).unwrap();
    let violations = run(&cfg, dir.path());

    assert_eq!(violations.len(), 1);
    assert!(violations.iter().any(|v| v.stem == "BadName"));
}

#[test]
#[ignore = "Resolving globs in 'dirs' is planned for Issue #2 but not yet implemented"]
fn run_resolves_glob_patterns_in_dirs() {
    let dir = TempDir::new().unwrap();
    scaffold(
        dir.path(),
        &["packages/auth/src/BadName.rs", "packages/ui/src/BadName.rs"],
    );

    let manifest = write_manifest(
        dir.path(),
        r#"
[[package.metadata.convention-lint.checks]]
dirs = ["packages/*/src"]
include = ["*.rs"]
format = "snake_case"
"#,
    );

    let cfg = load_config(&manifest).unwrap();
    let violations = run(&cfg, dir.path());

    assert_eq!(violations.len(), 2);
}

#[test]
fn run_with_recursive_false_only_checks_top_level() {
    let dir = TempDir::new().unwrap();
    scaffold(dir.path(), &["top_level_Bad.rs", "subdir/nested_Bad.rs"]);

    let manifest = write_manifest(
        dir.path(),
        r#"
[[package.metadata.convention-lint.checks]]
dirs = ["."]
include = ["*.rs"]
format = "snake_case"
recursive = false
"#,
    );

    let cfg = load_config(&manifest).unwrap();
    let violations = run(&cfg, dir.path());

    assert_eq!(
        violations.len(),
        1,
        "Should only find 1 violation in top-level directory"
    );
    assert_eq!(violations[0].stem, "top_level_Bad");
}

#[test]
fn run_include_glob_is_case_sensitive_by_default() {
    let dir = TempDir::new().unwrap();
    scaffold(dir.path(), &["BadName.RS", "GoodName.rs"]);

    let manifest = write_manifest(
        dir.path(),
        r#"
[[package.metadata.convention-lint.checks]]
dirs = ["."]
include = ["*.rs"]
format = "snake_case"
"#,
    );

    let cfg = load_config(&manifest).unwrap();
    let violations = run(&cfg, dir.path());

    assert!(
        !violations
            .iter()
            .map(|v| v.stem.as_str())
            .any(|x| x == "BadName"),
        "BadName should be excluded"
    );
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
        r#"
[[package.metadata.convention-lint.checks]]
dirs = ["."]
include = ["*.idl"]
format = "snake_case"
"#,
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
        r#"
[[package.metadata.convention-lint.checks]]
dirs = ["."]
include = ["*.idl"]
format = "snake_case"
"#,
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

// ---------------------------------------------------------------------------
// run — logic & filtering
// ---------------------------------------------------------------------------

#[test]
fn run_respects_gitignore() {
    let dir = TempDir::new().unwrap();

    fs::create_dir(dir.path().join(".git")).unwrap();

    scaffold(dir.path(), &["ignored_File.rs", "valid_file.rs"]);
    fs::write(dir.path().join(".gitignore"), "ignored_File.rs").unwrap();

    let manifest = write_manifest(
        dir.path(),
        r#"
[[package.metadata.convention-lint.checks]]
dirs = ["."]
include = ["*.rs"]
format = "snake_case"
"#,
    );

    let cfg = load_config(&manifest).unwrap();
    let violations = run(&cfg, dir.path());

    assert!(
        violations.is_empty(),
        "File in .gitignore should be skipped, but found: {violations:#?}"
    );
}

#[test]
fn run_handles_dotfiles_gracefully() {
    let dir = TempDir::new().unwrap();
    scaffold(dir.path(), &[".hidden_config"]);

    let manifest = write_manifest(
        dir.path(),
        r#"
[[package.metadata.convention-lint.checks]]
dirs = ["."]
include = ["*.txt"]
format = "snake_case"
"#,
    );

    let cfg = load_config(&manifest).unwrap();
    let violations = run(&cfg, dir.path());

    assert!(
        violations.is_empty(),
        "dotfile should not match *.txt include, got: {violations:#?}"
    );
}

#[test]
fn run_reports_multiple_violations_with_same_stem() {
    let dir = TempDir::new().unwrap();
    scaffold(dir.path(), &["a/BadName.rs", "b/BadName.rs"]);

    let manifest = write_manifest(
        dir.path(),
        r#"
[[package.metadata.convention-lint.checks]]
dirs = ["."]
include = ["*.rs"]
format = "snake_case"
"#,
    );

    let cfg = load_config(&manifest).unwrap();
    let violations = run(&cfg, dir.path());

    assert_eq!(violations.len(), 2);
    let paths: Vec<String> = violations
        .iter()
        .map(|v| v.path.to_string_lossy().into_owned())
        .collect();
    assert!(paths.iter().any(|p| p.contains("a/BadName.rs")));
    assert!(paths.iter().any(|p| p.contains("b/BadName.rs")));
}
