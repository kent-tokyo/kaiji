# kaiji — CJK Fuzzy Match & Normalization Engine

**kaiji** is a high-performance Rust library that solves the "variant character" problem endemic to
Japanese, Chinese, and Korean text processing.

> 斎藤 ≡ 齋藤 ≡ 齊藤 &nbsp;·&nbsp; 渡辺 ≡ 渡邊 ≡ 渡邉 &nbsp;·&nbsp; 吉野家 ≡ 𠮷野家 &nbsp;·&nbsp; 広島 ≡ 廣島

**[▶ Try in browser →](https://kent-tokyo.github.io/kaiji/playground/)**

---

## The Problem

Standard Unicode normalization (NFC / NFD) fixes *form* differences but leaves *semantic* CJK
variants untouched. The result: the same name or word stored in two systems looks different to a
computer, even though every human reader knows they are identical.

| Symptom | Root Cause |
|---------|-----------|
| eKYC rejects a customer whose name uses old kanji | OCR output ≠ user input |
| RAG/LLM embeds 齋藤 and 斎藤 as separate tokens | Tokenizer sees different bytes |
| Search returns 0 results for "𠮷野家" | EC database stored "吉野家" |
| Property registry cannot be joined to CRM | 登記簿 uses 旧字体, CRM uses 新字体 |

Each team works around it with bespoke regex lists or giant `REPLACE()` chains. There is no
authoritative, fast, cross-language core — until now.

---

## Performance

Benchmarked on Apple M2 (single thread, `cargo bench`, optimized release build).

### Bulk normalization throughput

| Batch size | Time | Throughput |
|-----------|------|-----------|
| 1,000 rows | 42 µs | ~250 MiB/s |
| 100,000 rows | 4.25 ms | ~251 MiB/s |
| **1,000,000 rows** | **42 ms** | **~252 MiB/s** |

> Throughput is flat across all batch sizes — the library scales linearly with no overhead.
> Extrapolating: **10 million rows ≈ 420 ms**.

### Single-string latency

| Input | Scenario | Latency |
|-------|---------|---------|
| 1 char (斉) | already canonical — zero allocation | **5.8 ns** |
| 1 char (齋) | variant fold required | **22 ns** |
| 4 chars (斉藤一郎) | clean input | **17 ns** |
| 4 chars (齋藤一郎) | 1 variant to fold | **50 ns** |
| 30 chars | clean input | **132 ns** |
| 30 chars | mixed variants | **236 ns** |
| 5,000 chars | clean | **29 µs** |
| 5,000 chars | mixed variants | **36 µs** |

---

## Why kaiji over existing tools

| Tool | What it covers | JP variants | CN variants | IVS | WASM | Language |
|------|---------------|:-----------:|:-----------:|:---:|:----:|----------|
| [mojimoji](https://github.com/studio-ousia/mojimoji) | Fullwidth ↔ halfwidth only | ✗ | ✗ | ✗ | ✗ | Python |
| [jaconv](https://github.com/ikegami-yukino/jaconv) | Kana + historical kana + romaji (Python only) | ✗ | ✗ | ✗ | ✗ | Python |
| ja_cvu_normalizer | JP variant chars (dict-based) | △ partial | ✗ | ✗ | ✗ | Python |
| normalize-japanese-addresses | JP addresses only | ✗ | ✗ | ✗ | ✓ | JS/TS |
| [OpenCC](https://github.com/BYVoid/OpenCC) | CN simplified ↔ traditional | ✗ | ✓ | ✗ | △ heavy | C++/Python/Node |
| ICU / CLDR | Unicode NFC/NFKC only | ✗ | ✗ | ✗ | ✗ | C++/Java |
| **kaiji** | **Variants + width + kana + historical kana + romaji + IVS + corpus search** | **✓ 446+** | **✓** | **✓** | **✓ lightweight** | **Rust/Python/WASM/CLI** |

kaiji is the first library to unify all of these capabilities in a single, compile-to-anywhere Rust core.

---

## What kaiji Solves

| Capability | Description |
|-----------|-------------|
| **Variant folding** | Maps 300+ old/traditional/variant CJK characters to their canonical new forms (JIS X 0213, 人名用漢字, Traditional Chinese) |
| **IVS stripping** | Removes Ideographic Variation Sequences (U+E0100–U+E01EF) that are invisible in most fonts but cause byte-level mismatches |
| **Width normalization** | Fullwidth ASCII (ＡＢＣ→ABC), halfwidth katakana (ｶﾞ→ガ) with dakuten composition |
| **Zero-copy fast path** | Input with no variants is returned as a borrowed `&str` slice — no allocation |
| **Fuzzy matching** | `matches("齋藤", "斎藤")` → `true` in a single call |
| **Kana normalization** | Hiragana↔Katakana conversion — `ひらがな→ヒラガナ` or `カタカナ→かたかな` |
| **Historical kana** | Obsolete kana to modern: ゐ→い, ゑ→え, を→お, ぢ→じ, づ→ず / ヰ→イ, ヱ→エ, ヲ→オ, ヂ→ジ, ヅ→ズ |
| **Romaji conversion** | Kana → Modified Hepburn romaji with long-vowel collapse: `サトウ→"sato"`, `トウキョウ→"tokyo"` |
| **Katakana halfwidth** | Fullwidth katakana → halfwidth with dakuten decomposition: ガ→ｶﾞ, パ→ﾊﾟ |
| **Modular pipeline** | Enable only what you need via `NormalizerConfig` or the builder API |

---

## Quick Start

**First-class bindings** (publish to pip / npm / crates.io):
[Rust](#rust) · [Python](#python) · [JavaScript / TypeScript](#javascript--typescript-webassembly) · [CLI](#cli)

**Advanced bindings** (require building the Rust native library locally):
[Go](#go) · [Java / Kotlin](#java--kotlin)

### Rust

```toml
[dependencies]
kaiji = { version = "0.1", features = ["japanese"] }
```

```rust
use kaiji::{matches_default, normalize_default, Normalizer};

// Simple fuzzy name matching
assert!(matches_default("斎藤", "齋藤").unwrap());     // 斎 vs 齋
assert!(matches_default("渡辺", "渡邊").unwrap());     // 辺 vs 邊
assert!(matches_default("𠮷野家", "吉野家").unwrap()); // tsuchi-yoshi vs kichi

// Normalization (zero-copy when input is already canonical)
let s = normalize_default("齋藤一郎").unwrap();
assert_eq!(s, "斉藤一郎");
```

### Python

```bash
pip install kaiji
```

```python
import kaiji

kaiji.normalize("齋藤")              # "斉藤"
kaiji.matches("斎藤", "齋藤")        # True
kaiji.similarity_score("斎藤", "齋藤")  # 1.0

n = kaiji.Normalizer(width_normalization=True)
n.normalize("ＡＢＣ齋藤")           # "ABC斉藤"

# Batch normalization (pandas-friendly)
kaiji.normalize_batch(["齋藤", "渡邊", "𠮷野家"])
# → ["斉藤", "渡辺", "吉野家"]

# Corpus index — fuzzy search over N records
idx = kaiji.Index(["斎藤一郎", "渡辺花子", "佐藤次郎"])
hits = idx.search("齋藤一郎", 0.9)   # threshold 0.0–1.0
hits[0].original  # "斎藤一郎"
hits[0].score     # 1.0
```

### CLI

```bash
# macOS (Homebrew)
brew tap kent-tokyo/kaiji
brew install kaiji

# Cross-platform (Rust)
cargo install kaiji-cli
```

```bash
# stdin → stdout normalization
echo "齋藤一郎" | kaiji normalize
# 斉藤一郎

# with fullwidth conversion
echo "ＡＢＣ齋藤" | kaiji normalize --width
# ABC斉藤

# fuzzy match (exit 0 = match, exit 1 = no match)
kaiji match "斎藤" "齋藤" && echo "matched"

# similarity score
kaiji score "斎藤一郎" "斉藤二郎"
# 0.9333

# batch normalization with tsv or json output
cat names.txt | kaiji normalize --format tsv
cat names.txt | kaiji normalize --format json
```

### JavaScript / TypeScript (WebAssembly)

```bash
npm install kaiji-wasm
```

```js
import init, { normalize, matches, similarity_score, Normalizer } from "kaiji-wasm";

await init();

normalize("齋藤");                  // "斉藤"
matches("斎藤", "齋藤");            // true
similarity_score("斎藤", "齋藤");   // 1.0

const n = new Normalizer(true, true, true, false);
n.normalize("ＡＢＣ齋藤");          // "ABC斉藤"
```

### Go

> **Note:** This binding requires building the Rust C shared library locally before use.
> See the instructions below.

```bash
# 1. Build the native library
cargo build --release --manifest-path crates/kaiji-c/Cargo.toml

# 2. Add the Go module
go get github.com/kent-tokyo/kaiji/bindings/go
```

```go
import (
    "fmt"
    kaiji "github.com/kent-tokyo/kaiji/bindings/go"
)

// Set CGO_LDFLAGS to point to the compiled library before building.
// export CGO_LDFLAGS="-L/path/to/crates/kaiji-c/target/release"

result, _ := kaiji.Normalize("齋藤")   // "斉藤"
matched, _ := kaiji.Matches("斎藤", "齋藤")  // true
score, _   := kaiji.Similarity("斎藤", "齋藤") // 1.0
fmt.Println(result, matched, score)
```

### Java / Kotlin

> **Note:** This binding requires building the Rust JNI shared library locally before use.
> See [bindings/java/README.md](bindings/java/README.md) for full setup instructions.

### Builder API (recommended, Rust)

```rust
use kaiji::Normalizer;

let n = Normalizer::builder()
    .width_normalization(true)  // ＡＢＣ → ABC, ｶﾞ → ガ
    .fold_variants(true)        // 齋 → 斉
    .strip_ivs(true)            // remove invisible variation selectors
    .build();

assert_eq!(n.normalize("ＡＢＣ齋藤").unwrap(), "ABC斉藤");
assert!(n.matches("渡辺一郎", "渡邊一郎").unwrap());

// Kana normalization (jaconv replacement)
let n = Normalizer::builder().kana_to_katakana(true).build();
assert_eq!(n.normalize("ひらがな").unwrap(), "ヒラガナ");

let n = Normalizer::builder().kana_to_hiragana(true).build();
assert_eq!(n.normalize("ラーメン").unwrap(), "らーめん");
```

### Corpus index (N-record fuzzy search)

```toml
[dependencies]
kaiji = { version = "0.1", features = ["japanese", "index"] }
```

```rust
use kaiji::{KaijiIndex, NormalizerConfig};

// Build once — all corpus strings are normalised and stored in an FST
let corpus = vec![
    "斎藤一郎".to_string(),
    "渡辺花子".to_string(),
    "佐藤次郎".to_string(),
];
let index = KaijiIndex::build(corpus, NormalizerConfig::default())?;

// Query — variant-form input still matches the canonical entry
let hits = index.search("齋藤一郎", 0.9)?;
// hits[0].original == "斎藤一郎", hits[0].score == 1.0
```

Strings that normalise to the same form share a slot — both originals are returned when that slot matches. Results are sorted by Jaro-Winkler score descending.

### Cargo features

```toml
[dependencies]
kaiji = { version = "0.1", features = ["japanese"] }          # default
kaiji = { version = "0.1", features = ["japanese", "nfkc"] }  # + NFKC normalization
kaiji = { version = "0.1", features = ["japanese", "index"] }  # + FST corpus index
kaiji = { version = "0.1", features = ["full"] }               # all features (excl. index)
```

---

## Use Cases

### 1. NLP / LLM Pre-processing (Data Engineers, ML Engineers)

Normalise millions of rows before tokenisation so that variant-form characters don't produce
duplicate embeddings or mislead your RAG retrieval pipeline. At **~250 MiB/s**, a 10 million-row
CSV of Japanese names normalises in under half a second.

### 2. eKYC & AML (FinTech / Security Engineers)

Match a user-entered name against an OCR-read identity document even when they use different
kanji variants. Eliminate expensive manual review queues caused by byte-level mismatches.

### 3. Property Registry Matching (PropTech)

Join 旧字体-heavy 登記簿 records to modern CRM / GIS systems without maintaining a forest of
`REPLACE()` SQL functions that kill database performance.

### 4. E-Commerce Search (Frontend / Search Engineers)

Compile kaiji to WebAssembly and run it client-side: silently normalise the search query before
it hits your index, eliminating "0 results" failures from variant-character input.

---

## Architecture

```
src/
├── lib.rs          — public re-exports
├── error.rs        — CjkFuzzyError, Result<T>
├── config.rs       — NormalizerConfig (#[non_exhaustive])
├── normalizer.rs   — Normalizer builder + 4-stage pipeline runner
├── width.rs        — Stage 1: fullwidth↔halfwidth, halfwidth-kana, NFKC
├── variants.rs     — Stage 2: static OnceLock HashMap (300+ variant→canonical)
├── normalize.rs    — Stage 2: normalize() → Cow<'a, str> (zero-copy fast path)
├── matcher.rs      — matches() / matches_default()
└── index.rs        — KaijiIndex: FST corpus index + Jaro-Winkler search (`index` feature)
```

**Pipeline stages:**

| Stage | What it does | Feature gate |
|-------|-------------|-------------|
| 1a | Fullwidth ASCII → halfwidth, halfwidth kana → fullwidth (dakuten composition) | `width_normalization: true` |
| 1b | Unicode NFKC normalization | `nfkc` feature + `nfkc: true` |
| 2 | IVS strip + CJK semantic variant folding | `strip_ivs`, `fold_variants` |
| 3 | Word-level context conversion (OpenCC-style) | *planned — `chinese` feature* |
| 4 | Domain rules (address normalization etc.) | *planned* |

The variant map is built once at first call via `std::sync::OnceLock` and shared across threads
with no further locking. `normalize()` borrows the input unchanged when no substitution is needed,
avoiding any heap allocation on the hot path.

---

## Roadmap

| Status | Item |
|--------|------|
| [done]    | Stage 2: variant folding + IVS strip |
| [done]    | Stage 1: width normalization (fullwidth, halfwidth kana, dakuten) |
| [done]    | `Normalizer` builder API |
| [done]    | Cargo features (`japanese`, `chinese`, `nfkc`, `full`) |
| [done]    | Variant dictionary expanded to 446+ character families |
| [done]    | `similarity_score()` (Jaro-Winkler on normalized strings) |
| [done]    | Python bindings via PyO3 (`pip install kaiji`) |
| [done]    | WebAssembly build via wasm-bindgen (`npm install kaiji-wasm`) |
| [done]    | `KaijiIndex` — FST corpus index + Jaro-Winkler fuzzy search (`index` feature) |
| [done]    | Japanese address normalization — 漢数字→Arabic, 丁目/番/号 unification (`address` feature) |
| [done]    | Python bindings — `normalize_batch()`, `Index` class, `SearchHit`, type stubs (.pyi) |
| [done]    | WASM `KaijiIndex` + `SearchHit` — JavaScript corpus search |
| [done]    | Browser Playground — GitHub Pages |
| [done]    | Homebrew formula — `brew tap kent-tokyo/kaiji` |
| [done]    | CLI (`kaiji-cli`) — `normalize`, `match`, `score` subcommands |
| [planned] | Stage 3: OpenCC-style word-level conversion (`chinese` feature) |

See [`tasks/todo.md`](tasks/todo.md) for the detailed task list.

---

## Building & Testing

```bash
cargo build
cargo test
cargo test --features nfkc     # with NFKC normalization
cargo test --features index    # with FST corpus index
cargo clippy --all-targets -- -D warnings
cargo fmt
cargo bench                    # requires criterion
```

---

## Why "kaiji"?

**解字 (kaiji, かいじ)** — *"to analyze and resolve a character to its canonical form."*

> **K**anji **A**nalysis and **I**VS **J**ormalization **I**ngine

The name comes from *Shuowen Jiezi* (説文解字, "Explaining Simple and Analyzing Compound Characters"),
the first systematic Chinese character dictionary, compiled by Xu Shen (許慎) around 100 CE.
In that work, each entry starts from a character as it appears in one written form and **resolves it
back to its authoritative structure and meaning** — stripping away regional or era-specific variation
to arrive at the single canonical reading.

That is exactly what this library does: it takes any written form of a CJK character — old kanji,
traditional Chinese, IVS glyph, fullwidth variant — and resolves it to its canonical code point,
so that `齋藤` and `斎藤` are finally the same string to a computer.

---

## License

MIT OR Apache-2.0
