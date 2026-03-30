# convention-lint

[<img alt="github" src="https://img.shields.io/badge/github-jaroslawroszyk/convention--lint-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/jaroslawroszyk/convention-lint)
[<img alt="crates.io" src="https://img.shields.io/crates/v/convention-lint.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/convention-lint)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-convention--lint-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/convention-lint)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/roszyk/convention-lint/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/jaroslawroszyk/convention-lint/actions?query=branch%3Amain)
[<img alt="license" src="https://img.shields.io/badge/license-MIT%2FApache--2.0-blue?style=for-the-badge" height="20">](#license)

A file-naming convention linter for Rust projects. Configure it once in `Cargo.toml`, run it as a Cargo subcommand.

Uses the `ignore` crate for parallel directory traversal (same as ripgrep), so it respects `.gitignore` and skips hidden files out of the box. Errors are printed in `rustc`/`clippy` style.

---

## Installation

```sh
cargo install convention-lint
```

The binary is named `cargo-convention-lint`, so Cargo picks it up automatically as a subcommand:

```sh
cargo convention-lint
```

---

## Configuration

Each check is a `[[...checks]]` entry under `package.metadata.convention-lint` (or `workspace.metadata.convention-lint`). Every entry specifies **which directories** to scan, **which files** to include/exclude, and **what naming convention** to enforce.

### Basic example

```toml
[[package.metadata.convention-lint.checks]]
dirs    = ["src"]
include = ["*.rs"]
format  = "snake_case"
```

### Fields

| Field       | Required | Default | Description |
|-------------|----------|---------|-------------|
| `dirs`      | yes      | —       | Directories to scan. Supports glob patterns (`*`, `**`, `?`). |
| `include`   | no       | all files | Glob patterns for files to check (e.g. `["*.rs", "*.py"]`). |
| `exclude`   | no       | none    | Glob patterns for files to skip (takes priority over `include`). |
| `format`    | yes      | —       | Naming convention to enforce (see [supported conventions](#supported-conventions)). |
| `recursive` | no       | `true`  | When `false`, only direct children of each dir are checked. |

### Include / exclude filtering

**Only include** — check only `.rs` files:

```toml
[[package.metadata.convention-lint.checks]]
dirs    = ["some/dir", "../../some/dir2"]
include = ["*.rs"]
format  = "camelCase"
```

**Include + exclude** — check `.py` and `.sh` but skip `__init__.py`:

```toml
[[package.metadata.convention-lint.checks]]
dirs    = ["other/dir"]
include = ["*.py", "*.sh"]
exclude = ["__init__.py"]
format  = "PascalCase"
```

**Only exclude** — check everything except specific files:

```toml
[[package.metadata.convention-lint.checks]]
dirs    = ["other/dir3"]
exclude = ["the-only-exclude.txt"]
format  = "PascalCase"
```

### Globbed directories

The `dirs` field supports glob patterns so you can target many directories with a single rule:

```toml
# Single-level wildcard — matches packages/foo/tests, packages/bar/tests, etc.
[[package.metadata.convention-lint.checks]]
dirs   = ["packages/*/tests"]
include = ["*.rs"]
format  = "snake_case"

# Recursive wildcard — matches src/flow, src/a/flow, src/a/b/flow, etc.
[[package.metadata.convention-lint.checks]]
dirs   = ["src/**/flow"]
include = ["*.rs"]
format  = "snake_case"
```

### Non-recursive search

By default every directory is scanned recursively. Set `recursive = false` to check only the direct children:

```toml
[[package.metadata.convention-lint.checks]]
dirs      = ["dir1/tests"]
recursive = false
format    = "snake_case"
# dir1/tests/foo.rs        — checked
# dir1/tests/sub/bar.rs    — skipped
```

### Workspace config

Put the config in the root `Cargo.toml` under `workspace.metadata`:

```toml
[[workspace.metadata.convention-lint.checks]]
dirs    = ["crates/*/src"]
include = ["*.rs"]
format  = "snake_case"

[[workspace.metadata.convention-lint.checks]]
dirs    = ["proto"]
include = ["*.proto"]
format  = "snake_case"
```

Then run:

```sh
cargo convention-lint
# or point at a specific manifest:
cargo convention-lint --manifest-path path/to/Cargo.toml
```

Exit code is `0` if everything passes, `1` if there are violations.

> Note Rule Merging: If you define checks in both workspace.metadata and package.metadata, the linter will combine them. This allows you to set global rules for the whole project and specific rules for individual crates.
---

## Supported conventions

| Name                  | Example        |
|-----------------------|----------------|
| `snake_case`          | `my_service`   |
| `CamelCase`           | `MyService`    |
| `camelCase`           | `myService`    |
| `SCREAMING_SNAKE_CASE`| `MY_CONSTANT`  |
| `kebab-case`          | `my-service`   |

`PascalCase` is accepted as an alias for `CamelCase`.

---

## Output

```
error[convention]: `src/idl/MyService.idl` — stem `MyService` does not follow snake_case convention
error[convention]: `src/idl/badName.idl` — stem `badName` does not follow snake_case convention

convention-lint: found 2 naming violation(s)
```

---

## Testing

`tests/fixtures/` contains two small projects you can run against directly:

```
tests/fixtures/
├── pass/          ← all files conform → exit 0
│   ├── Cargo.toml
│   ├── idl/
│   │   ├── my_service.idl
│   │   └── order_processor.idl
│   └── src/
│       └── my_module.rs
└── fail/          ← intentional violations → exit 1
    ├── Cargo.toml
    ├── idl/
    │   ├── my_service.idl    ✓
    │   ├── MyService.idl     ✗  (should be snake_case)
    │   └── another_Bad.idl   ✗
    └── src/
        ├── OrderProcessor.rs ✓
        └── bad_module.rs     ✗  (should be CamelCase)
```

```sh
cargo run -- convention-lint --manifest-path tests/fixtures/pass/Cargo.toml
cargo run -- convention-lint --manifest-path tests/fixtures/fail/Cargo.toml
```

Full test suite:

```sh
cargo test
```

---

## CI

### GitHub Actions

```yaml
- name: Install convention-lint
  uses: taiki-e/install-action@v2
  with:
    tool: convention-lint

- name: Check naming conventions
  run: cargo convention-lint
```

### Pre-commit hook

```sh
#!/bin/sh
cargo convention-lint || exit 1
```

---

## Library usage

The crate also works as a library if you want to embed it in a build script or another tool:

```toml
[dependencies]
convention-lint = "0.2"
```

```rust
use convention_lint::{config::load_config, lint::run};
use std::path::Path;

fn main() {
    let cfg = load_config(Path::new("Cargo.toml")).expect("failed to load config");
    let violations = run(&cfg, Path::new("."));

    for v in &violations {
        eprintln!("{v}");
    }

    if !violations.is_empty() {
        std::process::exit(1);
    }
}
```

Public API:

| Item | Description |
|------|-------------|
| `convention_lint::Convention` | enum of supported conventions |
| `convention_lint::Error` | error variants from config loading |
| `convention_lint::Violation` | a single naming violation |
| `convention_lint::config::load_config` | parse config from a `Cargo.toml` path |
| `convention_lint::lint::run` | walk the filesystem and collect violations |

Full docs on [docs.rs/convention-lint](https://docs.rs/convention-lint).

---

## License

- [MIT](LICENSE-MIT)
- [Apache 2.0](LICENSE-APACHE),
