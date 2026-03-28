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

Add a `[package.metadata.convention-lint]` section to your `Cargo.toml`. Each key is a file extension mapped to a convention name.

**Single package:**

```toml
[package.metadata.convention-lint]
rs = "CamelCase"

[package.metadata.convention-lint.dirs]
rs = ["src/models"]
```

**Workspace** — put the config in the root `Cargo.toml` under `[workspace.metadata]`:

```toml
[workspace.metadata.convention-lint]
rs    = "snake_case"
idl   = "snake_case"
proto = "snake_case"

[workspace.metadata.convention-lint.dirs]
idl = ["src/idl", "proto"]
# extensions without a dirs entry are scanned recursively
```

Then run:

```sh
cargo convention-lint
# or point at a specific manifest:
cargo convention-lint --manifest-path path/to/Cargo.toml
```

Exit code is `0` if everything passes, `1` if there are violations.

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
convention-lint = "0.1"
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
