# Changelog

## [0.2.0] - 2026-05-17

### Added

- **`KaijiIndex`** (`index` feature, `src/index.rs`) — FST-backed corpus index; `build(iter, config)` + `search(query, threshold) -> Vec<SearchHit>` sorted by Jaro-Winkler score
- **Address normalization Stage 4** (`address` feature, `src/address.rs`) — converts kanji numerals to Arabic (三丁目→3丁目, 二十三番地→23番地); controlled by `NormalizerConfig::address_normalization`
- **Python bindings** (`bindings/python/`) — added `normalize_batch(list[str])`, `Index` class, `SearchHit` class; type stubs (`.pyi` + `py.typed`, PEP 561); maturin wheel matrix for PyPI
- **WASM `KaijiIndex`** (`bindings/wasm/`) — `KaijiIndex` and `SearchHit` now exported to JavaScript; `search()` returns `js_sys::Array`
- **Browser Playground** (`docs/playground/`) — static HTML + Web-target WASM; deployed to GitHub Pages via `.github/workflows/pages.yml`
- **Homebrew formula** (`Formula/kaiji.rb`) — tap-installable CLI binary for macOS arm64 and x86_64
- **`chinese` feature** — added J-canonical→Simplified Chinese mappings (21 entries) so cross-form matching works when feature is enabled
- **Variant dictionary** expanded from ~150 to 446 entries (JIS X 0213, 人名用漢字, IVD representative characters)
- **Benchmarks** — `benches/index_bench.rs` (build + search at 100/1k/10k corpus sizes)
- **Release pipeline** (`.github/workflows/release.yml`) — tag-triggered multi-platform publish: crates.io, PyPI (manylinux + macOS universal2 + Windows), npm (`kaiji-wasm`), GitHub Releases (CLI binaries for Linux musl, macOS arm64/x86_64, Windows)

## [0.1.0] - 2026-05-17

### Added

- **Core normalization pipeline** (`src/normalize.rs`)
  - `normalize(input, config) -> Result<Cow<str>>` with zero-copy fast path
  - `normalize_default(input)` convenience wrapper
  - IVS selector stripping (U+E0100..=U+E01EF)
  - CJK semantic variant folding via static `OnceLock` HashMap

- **Variant dictionary** (`src/variants.rs`)
  - 150+ character family mappings: 斉/斎/齋/齊, 辺/邊/邉, 広/廣, 関/關, etc.
  - Includes 人名用異体字: 髙 (はしご高), 﨑 (たつさき)
  - `𠮷` (tsuchi-yoshi) → `吉` mapping

- **Stage 1: width normalization** (`src/width.rs`)
  - Fullwidth ASCII (U+FF01–U+FF5E) → halfwidth ASCII
  - Fullwidth space (U+3000) → halfwidth space
  - Halfwidth katakana (U+FF65–U+FF9F) → fullwidth katakana with look-ahead dakuten/handakuten composition
  - Optional Unicode NFKC normalization behind the `nfkc` Cargo feature

- **Normalizer builder API** (`src/normalizer.rs`)
  - `Normalizer::builder()` → `NormalizerBuilder` → `Normalizer`
  - `Normalizer::normalize()`, `Normalizer::matches()`, `Normalizer::similarity()`

- **Similarity scoring** (`src/similarity.rs`)
  - `similarity_score(a, b, config) -> Result<f32>` using Jaro-Winkler distance on normalized strings
  - Variant forms collapse before comparison: `similarity_score("斎藤", "齋藤", &cfg)` returns `1.0`

- **Fuzzy matching** (`src/matcher.rs`)
  - `matches(a, b, config) -> Result<bool>`
  - `matches_default(a, b) -> Result<bool>`

- **Cargo features**
  - `japanese` (default): Japanese variant dictionary
  - `chinese`: reserved for future OpenCC-style word conversion
  - `nfkc`: Unicode NFKC normalization via `unicode-normalization`
  - `full`: all features enabled

- **Benchmarks** (`benches/`)
  - `normalize_bench.rs`: single-char, short, medium, long-string latency groups
  - `normalize_bench.rs`: `bench_width` group — fullwidth ASCII, halfwidth kana dakuten, zero-copy fast path, full pipeline
  - `bulk_bench.rs`: 1k / 100k / 1M row throughput with `Throughput::Bytes` (~250 MiB/s)

- **CLI examples**
  - `examples/normalize_cli.rs`: stdin → stdout normalization with optional `--width` flag
  - `examples/match_names.rs`: tabular name-pair matching display

- **Error type** (`src/error.rs`)
  - `CjkFuzzyError` with `InvalidInput` and `NormalizationFailed` variants
  - `Result<T>` type alias

[0.1.0]: https://github.com/kent-tokyo/kaiji/releases/tag/v0.1.0
