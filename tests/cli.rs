//! End-to-end CLI tests.
//!
//! These tests invoke the compiled `cargo-convention-lint` binary against real
//! fixture projects under `tests/fixtures/` and assert on exit codes and
//! output — the same way a user would run `cargo convention-lint`.
//!
//! # Fixture layout
//!
//! Each fixture uses `[[package.metadata.convention-lint.checks]]` entries
//! with `dirs`, `include`, and `format` fields.
//!
//! ```text
//! tests/fixtures/
//! ├── pass/          ← all files conform → exit 0
//! │   ├── Cargo.toml
//! │   ├── idl/
//! │   │   ├── my_service.idl
//! │   │   └── order_processor.idl
//! │   └── src/
//! │       └── my_module.rs
//! └── fail/          ← intentional violations → exit 1
//!     ├── Cargo.toml
//!     ├── idl/
//!     │   ├── my_service.idl   (ok — snake_case)
//!     │   ├── MyService.idl    (violation: should be snake_case)
//!     │   └── another_Bad.idl  (violation: should be snake_case)
//!     └── src/
//!         ├── OrderProcessor.rs  (ok — CamelCase)
//!         └── bad_module.rs      (violation: should be CamelCase)
//! ```

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

/// Build a `Command` for the linter binary, pre-loaded with the `convention-lint`
/// subcommand argument that Cargo would normally inject.
fn linter() -> Command {
    let mut cmd = Command::cargo_bin("cargo-convention-lint")
        .expect("binary `cargo-convention-lint` not found — run `cargo build` first");
    cmd.arg("convention-lint");
    cmd
}

/// Resolve a path relative to the workspace root (where `cargo test` is run).
fn fixture(rel: &str) -> String {
    format!("tests/fixtures/{rel}/Cargo.toml")
}

// ---------------------------------------------------------------------------
// Happy path
// ---------------------------------------------------------------------------

#[test]
fn pass_fixture_exits_zero() {
    linter()
        .args(["--manifest-path", &fixture("pass")])
        .assert()
        .success();
}

#[test]
fn pass_fixture_reports_all_files_ok() {
    linter()
        .args(["--manifest-path", &fixture("pass")])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "all files follow configured naming conventions",
        ));
}

#[test]
fn pass_fixture_produces_no_error_lines() {
    linter()
        .args(["--manifest-path", &fixture("pass")])
        .assert()
        .success()
        .stdout(predicate::str::contains("error[convention]").not());
}

// ---------------------------------------------------------------------------
// Violation path
// ---------------------------------------------------------------------------

#[test]
fn fail_fixture_exits_nonzero() {
    linter()
        .args(["--manifest-path", &fixture("fail")])
        .assert()
        .failure();
}

#[test]
fn fail_fixture_reports_violation_count() {
    linter()
        .args(["--manifest-path", &fixture("fail")])
        .assert()
        .failure()
        .stderr(predicate::str::contains("naming violation(s)"));
}

#[test]
fn fail_fixture_lists_bad_idl_stems() {
    linter()
        .args(["--manifest-path", &fixture("fail")])
        .assert()
        .failure()
        .stderr(predicate::str::contains("MyService"))
        .stderr(predicate::str::contains("another_Bad"));
}

#[test]
fn fail_fixture_lists_bad_rs_stem() {
    linter()
        .args(["--manifest-path", &fixture("fail")])
        .assert()
        .failure()
        .stderr(predicate::str::contains("bad_module"));
}

#[test]
fn fail_fixture_does_not_flag_conformant_files() {
    // `my_service.idl` (snake_case ✓) and `OrderProcessor.rs` (CamelCase ✓)
    // must never appear in the violation output.
    let output = linter()
        .args(["--manifest-path", &fixture("fail")])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(
        !stdout.contains("my_service"),
        "`my_service` is conformant and must not be reported\n---\n{stdout}"
    );
    assert!(
        !stdout.contains("OrderProcessor"),
        "`OrderProcessor` is conformant and must not be reported\n---\n{stdout}"
    );
}

// ---------------------------------------------------------------------------
// Error handling
// ---------------------------------------------------------------------------

#[test]
fn missing_manifest_exits_nonzero_with_message() {
    linter()
        .args(["--manifest-path", "nonexistent/Cargo.toml"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error:"));
}

#[test]
fn missing_metadata_section_exits_nonzero() {
    use std::fs;
    let dir = tempfile::tempdir().unwrap();
    let empty_toml = dir.path().join("Cargo.toml");

    fs::write(
        &empty_toml,
        "[package]\nname = \"empty\"\nversion = \"0.1.0\"",
    )
    .unwrap();

    linter()
        .args(["--manifest-path", empty_toml.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error:"));
}

#[test]
fn workspace_metadata_cli_test() {
    let dir = tempfile::tempdir().unwrap();
    let toml_path = dir.path().join("Cargo.toml");

    std::fs::write(
        &toml_path,
        r#"[workspace]
members = ["crate_a"]

[[workspace.metadata.convention-lint.checks]]
dirs = ["crate_a/src"]
include = ["*.rs"]
format = "snake_case"
"#,
    )
    .unwrap();

    let src_dir = dir.path().join("crate_a/src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("BadFileName.rs"), "").unwrap();

    linter()
        .args(["--manifest-path", toml_path.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("BadFileName.rs"))
        .stderr(predicate::str::contains("does not follow snake_case"));
}
