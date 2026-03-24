# convention-lint

[![Crates.io](https://img.shields.io/crates/v/convention-lint)](https://crates.io/crates/convention-lint)
[![docs.rs](https://img.shields.io/docsrs/convention-lint)](https://docs.rs/convention-lint)
[![CI](https://github.com/roszyk/convention-lint/actions/workflows/ci.yml/badge.svg)](https://github.com/roszyk/convention-lint/actions)
[![License](https://img.shields.io/crates/l/convention-lint)](LICENSE-MIT)

A file-naming convention linter that you configure once in `Cargo.toml` and run
as a Cargo subcommand — or embed as a library in your own tooling.

---

## Installation

```sh
cargo install convention-lint
```

This installs the `cargo-convention-lint` binary into `~/.cargo/bin`.  Because
Cargo resolves subcommands by looking for `cargo-<name>` on `PATH`, the
installed binary is immediately usable as:

```sh
cargo convention-lint
```

---

## Quick start

Add a `[package.metadata.convention-lint]` section to your project's
`Cargo.toml`.  Each key is a file extension (without `.`) mapped to a
convention name:

```toml
[package.metadata.convention-lint]
idl = "snake_case"
rs  = "CamelCase"
proto = "snake_case"
```

Optionally restrict which directories are scanned per extension (paths are
relative to the manifest):

```toml
[package.metadata.convention-lint.dirs]
idl   = ["src/idl", "proto"]
proto = ["proto"]
# `rs` has no entry here → the whole project root is scanned
```

Then run:

```sh
cargo convention-lint
# or explicitly:
cargo convention-lint --manifest-path path/to/Cargo.toml
```

The linter exits with code `0` when all names are conformant, or `1` when
violations are found — making it suitable for CI pipelines.

---

## Supported conventions

| Identifier            | Example          | Description                        |
|-----------------------|------------------|------------------------------------|
| `snake_case`          | `my_service`     | All lowercase, underscores         |
| `CamelCase`           | `MyService`      | UpperCamelCase / PascalCase        |
| `camelCase`           | `myService`      | lowerCamelCase                     |
| `SCREAMING_SNAKE_CASE`| `MY_CONSTANT`    | All uppercase, underscores         |
| `kebab-case`          | `my-service`     | All lowercase, hyphens             |

`PascalCase` is accepted as an alias for `CamelCase`.

---

## Output format

Violations are printed in the same `error[…]: …` style used by `rustc` and
`clippy`, so they render correctly in most CI log viewers:

```
error[convention]: `src/idl/MyService.idl` — stem `MyService` does not follow snake_case convention
error[convention]: `src/idl/badName.idl` — stem `badName` does not follow snake_case convention

convention-lint: found 2 naming violation(s)
```

---

## CI integration

### GitHub Actions

```yaml
# .github/workflows/ci.yml
- name: Install convention-lint
  run: cargo install convention-lint

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

`convention-lint` exposes its full API as a library so you can embed it in
build scripts, proc-macros, or other Cargo plugins:

```toml
# Cargo.toml
[dependencies]
convention-lint = "0.1"
```

```rust
use convention_lint::{config::load_config, lint::run};

fn main() {
    let cfg = load_config(std::path::Path::new("Cargo.toml")).unwrap();
    let violations = run(&cfg, std::path::Path::new("."));

    for v in &violations {
        eprintln!("{v}");
    }

    std::process::exit(if violations.is_empty() { 0 } else { 1 });
}
```

The public API surface:

| Item | Description |
|------|-------------|
| `convention_lint::Convention` | Enum of all supported conventions |
| `convention_lint::Error` | All error variants from config loading |
| `convention_lint::Violation` | A single naming violation |
| `convention_lint::config::load_config` | Parse config from a `Cargo.toml` path |
| `convention_lint::lint::run` | Walk the filesystem and return violations |

See [docs.rs/convention-lint](https://docs.rs/convention-lint) for the full API
reference.

---

## License

Licensed under either of

- [MIT](LICENSE-MIT)
- [Apache 2.0](LICENSE-APACHE)

at your option.
