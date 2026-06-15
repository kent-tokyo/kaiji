# kaiji — CJK 模糊匹配与文本标准化引擎

**kaiji** 是一个高性能 Rust 库，专门解决日语、中文和韩语文本处理中普遍存在的"异体字问题"。

> 斎藤 ≡ 齋藤 ≡ 齊藤 &nbsp;·&nbsp; 渡辺 ≡ 渡邊 ≡ 渡邉 &nbsp;·&nbsp; 吉野家 ≡ 𠮷野家 &nbsp;·&nbsp; 広島 ≡ 廣島

**[▶ 在浏览器中体验 →](https://kent-tokyo.github.io/kaiji/playground/)**

---

## 问题说明

Unicode 标准规范化（NFC / NFD）只能修正字符"形式"上的差异，但对 CJK 语义级异体字（繁简、旧字形、IVS 异体字选择符）毫无作用。结果是：同一个姓名或词语在不同系统中看起来完全不同，尽管人类读者一眼就能识别它们是同一个字。

| 症状 | 根本原因 |
|------|---------|
| eKYC 因旧字形姓名被拒 | OCR 输出 ≠ 用户输入 |
| RAG/LLM 将"齋藤"和"斎藤"嵌入为不同 token | 分词器看到不同字节 |
| 搜索"𠮷野家"返回 0 结果 | EC 数据库存储的是"吉野家" |
| 不动产登记簿无法与 CRM 关联 | 登记簿使用旧字形，CRM 使用新字形 |

各团队用临时的正则表达式列表或大量 `REPLACE()` SQL 应对此问题。始终缺乏一个权威、快速、跨语言的核心解决方案——直到现在。

---

## 性能

在 Apple M2 上测试（单线程，`cargo bench`，优化 release 构建）。

### 批量标准化吞吐量

| 批次大小 | 耗时 | 吞吐量 |
|---------|------|--------|
| 1,000 行 | 42 µs | ~250 MiB/s |
| 100,000 行 | 4.25 ms | ~251 MiB/s |
| **1,000,000 行** | **42 ms** | **~252 MiB/s** |

> 吞吐量在所有批次大小下保持恒定——库的扩展完全线性，无额外开销。  
> 外推：**1000 万行 ≈ 420 ms**。

### 单字符串延迟

| 输入 | 场景 | 延迟 |
|------|------|------|
| 1 字符（斉） | 已是规范形——零分配 | **5.8 ns** |
| 1 字符（齋） | 需要异体字折叠 | **22 ns** |
| 4 字符（斉藤一郎） | 干净输入 | **17 ns** |
| 4 字符（齋藤一郎） | 1 个异体字需折叠 | **50 ns** |
| 30 字符 | 干净输入 | **132 ns** |
| 30 字符 | 混合异体字 | **236 ns** |
| 5,000 字符 | 干净 | **29 µs** |
| 5,000 字符 | 混合异体字 | **36 µs** |

---

## 与现有工具的对比

| 工具 | 覆盖范围 | CJK 异体字 | WASM | 跨语言 |
|------|---------|-----------|------|--------|
| [mojimoji](https://github.com/studio-ousia/mojimoji) | 仅全角↔半角 | ✗ | ✗ | 仅 Python |
| [jaconv](https://github.com/ikegami-yukino/jaconv) | 仅假名转换 | ✗ | ✗ | 仅 Python |
| [OpenCC](https://github.com/BYVoid/OpenCC) | 仅中文简繁转换 | ✗（不支持日语） | △ 重型 C++ | C++/Python/Node |
| **kaiji** | **异体字＋字宽＋IVS＋语料检索** | **✓ 300+ 字族** | **✓ 轻量** | **Rust/Python/WASM/CLI** |

kaiji 是第一个将所有这些功能统一在单一可编译到任何平台的 Rust 核心中的库。

---

## kaiji 解决的问题

| 功能 | 说明 |
|------|------|
| **异体字折叠** | 将 300+ 个旧/繁体/变体 CJK 字符映射到其规范新字形（JIS X 0213、人名用汉字、繁体中文） |
| **IVS 剥离** | 移除表意文字变体序列（U+E0100–U+E01EF），这些序列在大多数字体中不可见，但会导致字节级不匹配 |
| **字宽规范化** | 全角 ASCII（ＡＢＣ→ABC）、半角片假名（ｶﾞ→ガ）含浊音合成 |
| **零拷贝快速路径** | 无需转换的输入以借用 `&str` 切片返回——无分配 |
| **模糊匹配** | `matches("齋藤", "斎藤")` → `true` 一行调用 |
| **模块化管道** | 通过 `NormalizerConfig` 或构建器 API 仅启用所需功能 |

---

## 快速上手

### Rust

```toml
[dependencies]
kaiji = { version = "0.1", features = ["japanese"] }
```

```rust
use kaiji::{matches_default, normalize_default, Normalizer};

// 简单的模糊姓名匹配
assert!(matches_default("斎藤", "齋藤").unwrap());     // 斎 vs 齋
assert!(matches_default("渡辺", "渡邊").unwrap());     // 辺 vs 邊
assert!(matches_default("𠮷野家", "吉野家").unwrap()); // tsuchi-yoshi vs kichi

// 标准化（输入已是规范形时零拷贝）
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

# 批量标准化（兼容 pandas）
kaiji.normalize_batch(["齋藤", "渡邊", "𠮷野家"])
# → ["斉藤", "渡辺", "吉野家"]

# 语料库索引——N 条记录的模糊搜索
idx = kaiji.Index(["斎藤一郎", "渡辺花子", "佐藤次郎"])
hits = idx.search("齋藤一郎", 0.9)   # 阈值 0.0–1.0
hits[0].original  # "斎藤一郎"
hits[0].score     # 1.0
```

### CLI

```bash
# macOS (Homebrew)
brew tap kent-tokyo/kaiji
brew install kaiji

# 跨平台（Rust）
cargo install kaiji-cli
```

```bash
# 标准输入 → 标准输出标准化
echo "齋藤一郎" | kaiji normalize
# 斉藤一郎

# 含全角转换
echo "ＡＢＣ齋藤" | kaiji normalize --width
# ABC斉藤

# 模糊匹配（exit 0 = 匹配，exit 1 = 不匹配）
kaiji match "斎藤" "齋藤" && echo "matched"

# 相似度分数
kaiji score "斎藤一郎" "斉藤二郎"
# 0.9333

# 以 tsv 或 json 格式批量标准化
cat names.txt | kaiji normalize --format tsv
cat names.txt | kaiji normalize --format json
```

### JavaScript / TypeScript（WebAssembly）

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

### 构建器 API（推荐，Rust）

```rust
use kaiji::Normalizer;

let n = Normalizer::builder()
    .width_normalization(true)  // ＡＢＣ → ABC, ｶﾞ → ガ
    .fold_variants(true)        // 齋 → 斉
    .strip_ivs(true)            // remove invisible variation selectors
    .build();

assert_eq!(n.normalize("ＡＢＣ齋藤").unwrap(), "ABC斉藤");
assert!(n.matches("渡辺一郎", "渡邊一郎").unwrap());
```

### 语料库索引（N 条记录的模糊搜索）

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

标准化为相同形式的字符串共享一个槽位——当该槽位匹配时，两个原始字符串都会被返回。结果按 Jaro-Winkler 分数降序排列。

### Cargo 功能特性

```toml
[dependencies]
kaiji = { version = "0.1", features = ["japanese"] }          # 默认
kaiji = { version = "0.1", features = ["japanese", "nfkc"] }  # + NFKC 规范化
kaiji = { version = "0.1", features = ["japanese", "index"] }  # + FST 语料库索引
kaiji = { version = "0.1", features = ["full"] }               # 全部功能（不含 index）
```

---

## 使用场景

### 1. NLP / LLM 预处理（数据工程师、ML 工程师）

在分词前对数百万行数据进行标准化，防止异体字产生重复嵌入或误导 RAG 检索管道。以 **~250 MiB/s** 的速度，1000 万行日语姓名 CSV 在不到半秒内即可完成标准化。

### 2. eKYC 与 AML（金融科技 / 安全工程师）

即使用户输入与 OCR 识别的身份证件使用不同的汉字变体，也能实现姓名匹配。消除因字节级不匹配导致的昂贵人工审核队列。

### 3. 不动产登记簿匹配（PropTech）

将充满旧字形的登记簿记录与现代 CRM / GIS 系统关联，无需维护一大堆拖慢数据库性能的 `REPLACE()` SQL 函数。

### 4. 电商搜索（前端 / 搜索工程师）

将 kaiji 编译为 WebAssembly 并在客户端运行：在查询命中索引前静默规范化搜索词，消除因异体字输入导致的"0 结果"问题。

---

## 架构

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

**管道阶段：**

| 阶段 | 功能 | 功能开关 |
|------|------|---------|
| 1a | 全角 ASCII → 半角，半角片假名 → 全角（浊音合成） | `width_normalization: true` |
| 1b | Unicode NFKC 规范化 | `nfkc` feature + `nfkc: true` |
| 2 | IVS 剥离 + CJK 语义异体字折叠 | `strip_ivs`, `fold_variants` |
| 3 | 词级上下文转换（OpenCC 风格） | *计划中——`chinese` feature* |
| 4 | 领域规则（地址标准化等） | *计划中* |

异体字映射表在首次调用时通过 `std::sync::OnceLock` 构建，之后无需加锁即可跨线程共享。`normalize()` 在无需替换时借用输入原样返回，热路径上不会产生堆分配。

---

## 路线图

| 状态 | 项目 |
|------|------|
| [完成]    | Stage 2: 异体字折叠 + IVS 剥离 |
| [完成]    | Stage 1: 字宽规范化（全角、半角片假名、浊音合成） |
| [完成]    | `Normalizer` 构建器 API |
| [完成]    | Cargo features（`japanese`、`chinese`、`nfkc`、`full`） |
| [完成]    | 异体字词典扩展至 446+ 字族 |
| [完成]    | `similarity_score()`（基于规范化字符串的 Jaro-Winkler 分数） |
| [完成]    | Python 绑定（PyO3 / `pip install kaiji`） |
| [完成]    | WebAssembly 构建（wasm-bindgen / `npm install kaiji-wasm`） |
| [完成]    | `KaijiIndex` — FST 语料库索引 + Jaro-Winkler 模糊搜索（`index` feature） |
| [完成]    | 日语地址标准化——汉数字→阿拉伯数字，丁目/番/号统一（`address` feature） |
| [完成]    | Python 绑定——`normalize_batch()`、`Index` 类、`SearchHit`、类型存根（.pyi） |
| [完成]    | WASM `KaijiIndex` + `SearchHit`——JavaScript 语料库搜索 |
| [完成]    | 浏览器 Playground——GitHub Pages |
| [完成]    | Homebrew formula——`brew tap kent-tokyo/kaiji` |
| [完成]    | CLI（`kaiji-cli`）——`normalize`、`match`、`score` 子命令 |
| [计划中]  | Stage 3: OpenCC 风格词级转换（`chinese` feature） |

详细任务列表请参见 [`tasks/todo.md`](tasks/todo.md)。

---

## 构建与测试

```bash
cargo build
cargo test
cargo test --features nfkc     # 含 NFKC 规范化
cargo test --features index    # 含 FST 语料库索引
cargo clippy --all-targets -- -D warnings
cargo fmt
cargo bench                    # 需要 criterion
```

---

## 关于名称

**解字（かいじ）** — "分析字形，还原字的本义"。

该词源自许慎《说文解字》（东汉，约公元 100 年）——中国第一部系统性汉字字典。
《说文解字》的每个条目都从某种书写形式出发，追溯该字的正统结构与含义。
这正是 kaiji 库对 CJK 异体字所做的事情。

---

## 许可证

MIT OR Apache-2.0
