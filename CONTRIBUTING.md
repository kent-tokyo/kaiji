# Contributing to kaiji

Thank you for your interest in contributing! kaiji is a Rust library for CJK text normalization and fuzzy matching.

## Development Setup

```bash
git clone https://github.com/kent-tokyo/kaiji
cd kaiji
cargo build
cargo test
cargo test --features "index,address,chinese"  # all features
```

Minimum Rust version: **1.85** (edition 2024).

## Running Checks

Before opening a pull request, ensure all of these pass:

```bash
cargo test --all-features
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## Adding Variant Dictionary Entries

The variant character map lives in `src/variants.rs`. Entries follow the format:

```rust
('旧字', '新字'), // optional comment
```

Rules:
- The canonical (new) form must be the JIS X 0213 new-form for Japanese entries.
- For `chinese` feature entries, add to the `#[cfg(feature = "chinese")]` block only.
- Never add self-mappings (`('X', 'X')`).
- Add a test assertion in the nearest `spot_check_*` test function.
- Run `cargo test --features chinese` to verify no regressions.

## Updating Bindings

When you modify `NormalizerConfig` or the `Normalizer` builder API, update **all** of these:

| Binding | File |
|---------|------|
| Python | `bindings/python/src/lib.rs` + `bindings/python/python/kaiji.pyi` |
| WASM | `bindings/wasm/src/lib.rs` |
| CLI | `crates/kaiji-cli/src/main.rs` |

Go and Java bindings wrap the C ABI (`crates/kaiji-c/`) and typically do not need changes for new config fields unless you expose them through the C API.

## Pull Request Guidelines

- Include tests for any new functionality.
- Keep PRs focused — one feature or fix per PR.
- Update `CHANGELOG.md` under an `[Unreleased]` section.
- All CI checks must pass before merge.

## Reporting Issues

Please open a GitHub Issue with:
- Rust version (`rustc --version`)
- Operating system
- A minimal reproducible example
- Expected vs. actual output
