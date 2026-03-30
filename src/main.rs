//! CLI binary for `cargo convention-lint`. See the [`convention_lint`] crate for the full API.
use convention_lint::config::load_config;
use convention_lint::lint::run;
use std::path::{Path, PathBuf};
use std::process;

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();

    // When invoked as `cargo convention-lint`, cargo injects "convention-lint"
    // as the first argument.
    if args.first().map(String::as_str) == Some("convention-lint") {
        args.remove(0);
    }

    let manifest_path = args
        .iter()
        .position(|a| a == "--manifest-path")
        .and_then(|i| args.get(i + 1))
        .map_or_else(|| PathBuf::from("Cargo.toml"), PathBuf::from);

    let project_root = if manifest_path.is_absolute() {
        manifest_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    } else {
        manifest_path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    };

    let config = match load_config(&manifest_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}");
            process::exit(1);
        }
    };

    if config.rules.is_empty() {
        eprintln!("warning: no conventions configured in [package.metadata.convention-lint]");
        process::exit(1);
    }

    let violations = run(&config, &project_root);

    for v in &violations {
        eprintln!("{v}");
    }

    if violations.is_empty() {
        println!("convention-lint: all files follow configured naming conventions");
    } else {
        eprintln!(
            "\nconvention-lint: found {} naming violation(s)",
            violations.len()
        );
        process::exit(1);
    }
}
