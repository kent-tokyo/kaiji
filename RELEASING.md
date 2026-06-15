# Release Process

## Pre-release Checklist

- [ ] All tests pass: `cargo test --features "index,address,chinese"`
- [ ] Clippy clean: `cargo clippy --all-targets -- -D warnings`
- [ ] `CHANGELOG.md` updated — move `[Unreleased]` items to the new version
- [ ] Version bumped consistently in:
  - `Cargo.toml` (root)
  - `crates/kaiji-cli/Cargo.toml`
  - `bindings/python/Cargo.toml`
  - `bindings/wasm/Cargo.toml`
  - `bindings/wasm/package.json` (version field)
- [ ] `Cargo.lock` committed

## GitHub Secrets Required

| Secret | Where to get it |
|--------|----------------|
| `CARGO_REGISTRY_TOKEN` | https://crates.io/settings/tokens |
| `NPM_TOKEN` | https://www.npmjs.com/settings/tokens (Automation token) |
| PyPI | Use Trusted Publishing — no token needed, configure at https://pypi.org/manage/project/kaiji/settings/publishing/ |

Set secrets at: **GitHub repo → Settings → Secrets and variables → Actions**

## Triggering the Release

The release pipeline (`.github/workflows/release.yml`) fires on any `v*` tag push.

```bash
git tag v0.2.0
git push origin v0.2.0
```

This automatically:
1. Builds CLI binaries for Linux/macOS/Windows and uploads to GitHub Releases
2. Publishes `kaiji` to crates.io
3. Publishes `kaiji-cli` to crates.io
4. Builds Python wheels (manylinux, universal2, Windows) and publishes to PyPI
5. Builds WASM and publishes `kaiji-wasm` to npm

## Updating the Homebrew Formula

After the GitHub Release is created:

1. Download the release archive:
   ```bash
   curl -L https://github.com/kent-tokyo/kaiji/archive/refs/tags/v0.2.0.tar.gz -o kaiji-v0.2.0.tar.gz
   sha256sum kaiji-v0.2.0.tar.gz
   ```
2. For each CLI binary asset (arm64, x86_64), compute the SHA256 of the `.tar.gz` or binary.
3. Update `Formula/kaiji.rb` with the real SHA256 values and the new version/URL.
4. Push the updated formula to the Homebrew tap repository.

## Post-release Verification

```bash
# crates.io
cargo add kaiji && cargo test

# PyPI
pip install kaiji && python -c "import kaiji; print(kaiji.normalize('齋藤'))"

# npm
npm install kaiji-wasm

# Homebrew (after formula update)
brew tap kent-tokyo/kaiji && brew install kaiji
kaiji normalize <<< "齋藤一郎"
```
