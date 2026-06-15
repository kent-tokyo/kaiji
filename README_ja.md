# kaiji — CJK ファジー検索・テキスト正規化エンジン

**kaiji** は、日本語・中国語・韓国語のテキスト処理に蔓延する「異体字問題」を根本から解決する、
Rust 製の高性能ライブラリです。

> 斎藤 ≡ 齋藤 ≡ 齊藤 &nbsp;·&nbsp; 渡辺 ≡ 渡邊 ≡ 渡邉 &nbsp;·&nbsp; 吉野家 ≡ 𠮷野家 &nbsp;·&nbsp; 広島 ≡ 廣島

**[▶ ブラウザで試す →](https://kent-tokyo.github.io/kaiji/playground/)**

---

## 何を解決するのか

Unicode の標準正規化（NFC / NFD）は文字の「形」の揺れは修正できますが、**CJK の意味的な
異体字（旧字体・異体字・IVS）は一切修正しません。** その結果、同じ人名・地名・企業名でも、
システムによっては全く別の文字列として扱われてしまいます。

| 症状 | 根本原因 |
|------|---------|
| eKYC が旧字体の名前を弾く | OCR 出力 ≠ ユーザー入力の文字コード |
| RAG / LLM が「齋藤」と「斎藤」を別トークンとして埋め込む | 同一人物なのにベクトルが分離 |
| EC サイトで「𠮷野家」を検索すると 0 件 | DB には「吉野家」（つち吉と木吉の違い）で登録 |
| 不動産登記簿と CRM の名寄せができない | 登記簿は旧字体、CRM は新字体 |

各チームが場当たり的な正規表現リストや大量の `REPLACE()` SQL で対応してきました。
**「あらゆる言語から呼び出せる、決定版の高速コア」が存在しなかったのです。**

---

## パフォーマンス

Apple M2（シングルスレッド、`cargo bench`、release ビルド）での実測値。

### 一括正規化スループット

| バッチサイズ | 処理時間 | スループット |
|------------|---------|------------|
| 1,000 行 | 42 µs | ~250 MiB/s |
| 100,000 行 | 4.25 ms | ~251 MiB/s |
| **1,000,000 行** | **42 ms** | **~252 MiB/s** |

> スループットはバッチサイズによらず一定 — 完全な線形スケーリング。  
> 外挿: **1000万行 ≈ 0.42秒**。

### 単文字列レイテンシ

| 入力 | シナリオ | レイテンシ |
|------|---------|----------|
| 1文字（斉） | 変換不要・ゼロアロケーション | **5.8 ns** |
| 1文字（齋） | 異体字フォールド | **22 ns** |
| 4文字（斉藤一郎） | クリーン入力 | **17 ns** |
| 4文字（齋藤一郎） | 異体字1文字 | **50 ns** |
| 30文字 | クリーン | **132 ns** |
| 30文字 | 異体字混在 | **236 ns** |
| 5,000文字 | クリーン | **29 µs** |
| 5,000文字 | 異体字混在 | **36 µs** |

---

## 既存ツールとの比較

| ツール | カバー範囲 | CJK異体字 | WASM | クロス言語 |
|--------|----------|---------|------|----------|
| [mojimoji](https://github.com/studio-ousia/mojimoji) | 全角↔半角のみ | ✗ | ✗ | Python のみ |
| [jaconv](https://github.com/ikegami-yukino/jaconv) | かな変換のみ | ✗ | ✗ | Python のみ |
| [OpenCC](https://github.com/BYVoid/OpenCC) | 中国語簡体↔繁体のみ | ✗（日本語非対応） | △ 重い C++ | C++/Python/Node |
| **kaiji** | **異体字＋文字幅＋IVS＋コーパス検索** | **✓ 300字族以上** | **✓ 軽量** | **Rust/Python/WASM/CLI** |

kaiji は、これらすべての機能を単一の「どこでもコンパイル可能な」Rust コアに統合した初のライブラリです。

---

## kaiji が提供するもの

| 機能 | 説明 |
|------|------|
| **異体字フォールディング** | JIS X 0213・人名用漢字・繁体字対応の300字族以上を正準形に統合 |
| **IVS ストリップ** | U+E0100–U+E01EF の異体字セレクタを除去（フォントには見えないがバイト列は異なる） |
| **文字幅正規化** | 全角ASCII（ＡＢＣ→ABC）・半角カタカナ（ｶﾞ→ガ）の濁点合成つき変換 |
| **ゼロコピー高速パス** | 変換不要な入力は `&str` を借用して返す、ヒープアロケーションなし |
| **ファジーマッチ** | `matches("齋藤", "斎藤")` → `true` を1行で |
| **モジュラーパイプライン** | `NormalizerConfig` またはビルダー API で必要な処理だけを有効化 |

---

## クイックスタート

### Rust

```toml
[dependencies]
kaiji = { version = "0.1", features = ["japanese"] }
```

```rust
use kaiji::{matches_default, normalize_default, Normalizer};

// 異体字を含む名前のファジーマッチ
assert!(matches_default("斎藤", "齋藤").unwrap());     // 斎 vs 齋
assert!(matches_default("渡辺", "渡邊").unwrap());     // 辺 vs 邊
assert!(matches_default("𠮷野家", "吉野家").unwrap()); // つち吉 vs 木吉

// 正規化（変換不要の場合はゼロコピー）
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

# 一括正規化（pandas 対応）
kaiji.normalize_batch(["齋藤", "渡邊", "𠮷野家"])
# → ["斉藤", "渡辺", "吉野家"]

# コーパスインデックス — N件のファジー検索
idx = kaiji.Index(["斎藤一郎", "渡辺花子", "佐藤次郎"])
hits = idx.search("齋藤一郎", 0.9)   # しきい値 0.0–1.0
hits[0].original  # "斎藤一郎"
hits[0].score     # 1.0
```

### CLI

```bash
# macOS (Homebrew)
brew tap kent-tokyo/kaiji
brew install kaiji

# クロスプラットフォーム（Rust）
cargo install kaiji-cli
```

```bash
# 標準入力 → 標準出力 正規化
echo "齋藤一郎" | kaiji normalize
# 斉藤一郎

# 全角変換を含む
echo "ＡＢＣ齋藤" | kaiji normalize --width
# ABC斉藤

# ファジーマッチ（exit 0 = 一致、exit 1 = 不一致）
kaiji match "斎藤" "齋藤" && echo "matched"

# 類似スコア
kaiji score "斎藤一郎" "斉藤二郎"
# 0.9333

# tsv または json 形式で一括正規化
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
# 1. ネイティブライブラリをビルド
cargo build --release --manifest-path crates/kaiji-c/Cargo.toml

# 2. Go モジュールを追加
go get github.com/kent-tokyo/kaiji/bindings/go
```

```go
import (
    "fmt"
    kaiji "github.com/kent-tokyo/kaiji/bindings/go"
)

// CGO_LDFLAGS でコンパイル済みライブラリのパスを設定してからビルドしてください。
// export CGO_LDFLAGS="-L/path/to/crates/kaiji-c/target/release"

result, _ := kaiji.Normalize("齋藤")              // "斉藤"
matched, _ := kaiji.Matches("斎藤", "齋藤")       // true
score, _   := kaiji.Similarity("斎藤", "齋藤")    // 1.0
fmt.Println(result, matched, score)
```

### ビルダー API（推奨）

```rust
use kaiji::Normalizer;

let n = Normalizer::builder()
    .width_normalization(true)  // ＡＢＣ → ABC、ｶﾞ → ガ
    .fold_variants(true)        // 齋 → 斉
    .strip_ivs(true)            // 不可視の IVS セレクタを除去
    .build();

assert_eq!(n.normalize("ＡＢＣ齋藤").unwrap(), "ABC斉藤");
assert!(n.matches("渡辺一郎", "渡邊一郎").unwrap());
```

### コーパスインデックス（N件ファジー検索）

```toml
[dependencies]
kaiji = { version = "0.1", features = ["japanese", "index"] }
```

```rust
use kaiji::{KaijiIndex, NormalizerConfig};

// 初回構築時にコーパス全体を正規化して FST に格納
let corpus = vec![
    "斎藤一郎".to_string(),
    "渡辺花子".to_string(),
    "佐藤次郎".to_string(),
];
let index = KaijiIndex::build(corpus, NormalizerConfig::default())?;

// クエリ — 異体字形式でも正準形エントリとマッチ
let hits = index.search("齋藤一郎", 0.9)?;
// hits[0].original == "斎藤一郎", hits[0].score == 1.0
```

同じ正準形に正規化される文字列はスロットを共有し、スロットがヒットした際に元の文字列が両方返されます。
結果は Jaro-Winkler スコアの降順でソートされます。

### Cargo features

```toml
[dependencies]
kaiji = { version = "0.1", features = ["japanese"] }          # デフォルト
kaiji = { version = "0.1", features = ["japanese", "nfkc"] }  # + NFKC 正規化
kaiji = { version = "0.1", features = ["japanese", "index"] }  # + FST コーパスインデックス
kaiji = { version = "0.1", features = ["full"] }               # 全機能（index 除く）
```

---

## ユースケース

### 1. NLP / LLM 前処理（データエンジニア・ML エンジニア）

LLM のファインチューニングや RAG システム構築時、テキストの正規化は精度の要です。
異体字が混在したまま埋め込むと、同一人物が別のベクトルに分散してしまいます。

kaiji なら **~250 MiB/s のスループット** で一括処理。1000万行の日本語CSVも **約0.4秒** で完了します。

### 2. eKYC / AML（金融・セキュリティエンジニア）

ユーザー入力の氏名と、免許証 OCR の読み取り結果が異体字の違いで不一致になるケースを
自動解消します。人手による目視確認キューを大幅に削減できます。

### 3. 不動産登記簿の名寄せ（PropTech バックエンド）

旧字体だらけの登記簿データを、DB への投入前に kaiji で一括正規化。
`REPLACE()` を何十個も並べた遅い SQL から解放されます。

### 4. EC サイト検索の「0件ヒット」防止（フロントエンド / 検索エンジニア）

kaiji を WebAssembly にコンパイルしてブラウザで動かすと、ユーザーが検索ボタンを押した
瞬間にバックグラウンドで異体字を正規化してからサーバーに送信できます。

---

## アーキテクチャ

```
src/
├── lib.rs          — パブリック再エクスポート
├── error.rs        — CjkFuzzyError, Result<T>
├── config.rs       — NormalizerConfig (#[non_exhaustive])
├── normalizer.rs   — Normalizer ビルダー + 4ステージパイプライン
├── width.rs        — Stage 1: 全角↔半角、半角カナ、NFKC
├── variants.rs     — Stage 2: 静的 OnceLock HashMap（300字族以上）
├── normalize.rs    — Stage 2: normalize() → Cow<'a, str>（ゼロコピー最適化）
├── matcher.rs      — matches() / matches_default()
└── index.rs        — KaijiIndex: FST コーパスインデックス + Jaro-Winkler 検索（`index` feature）
```

**パイプラインステージ:**

| ステージ | 処理内容 | 有効化条件 |
|---------|---------|----------|
| 1a | 全角ASCII → 半角、半角カナ → 全角（濁点合成） | `width_normalization: true` |
| 1b | Unicode NFKC 正規化 | `nfkc` feature + `nfkc: true` |
| 2 | IVS 除去 + CJK 異体字フォールド | `strip_ivs`, `fold_variants` |
| 3 | 文脈依存単語変換（OpenCC 相当） | *予定 — `chinese` feature* |
| 4 | ドメイン特化ルール（住所正規化等） | *予定* |

異体字マップは初回呼び出し時に `std::sync::OnceLock` で構築され、以降はロックなしでスレッド間共有されます。
`normalize()` は変換不要な入力をそのまま借用して返すため、ホットパスでヒープアロケーションが発生しません。

---

## ロードマップ

| 状態 | 項目 |
|------|------|
| [完了]  | Stage 2: 異体字フォールド + IVS ストリップ |
| [完了]  | Stage 1: 文字幅正規化（全角/半角、濁点合成） |
| [完了]  | `Normalizer` ビルダー API |
| [完了]  | Cargo features（`japanese` / `chinese` / `nfkc` / `full`） |
| [完了]  | 異体字辞書を 446 字族以上に拡充 |
| [完了]  | `similarity_score()`（Jaro-Winkler 距離スコア） |
| [完了]  | Python バインディング（PyO3 / `pip install kaiji`） |
| [完了]  | WebAssembly ビルド（wasm-bindgen / `npm install kaiji-wasm`） |
| [完了]  | `KaijiIndex` — FST コーパスインデックス + Jaro-Winkler ファジー検索（`index` feature） |
| [完了]  | 住所正規化 — 漢数字→算用数字、丁目/番/号統一（`address` feature） |
| [完了]  | Python バインディング — `normalize_batch()`、`Index` クラス、`SearchHit`、型スタブ（.pyi） |
| [完了]  | WASM `KaijiIndex` + `SearchHit` — JavaScript コーパス検索 |
| [完了]  | ブラウザプレイグラウンド — GitHub Pages |
| [完了]  | Homebrew formula — `brew tap kent-tokyo/kaiji` |
| [完了]  | CLI（`kaiji-cli`） — `normalize`、`match`、`score` サブコマンド |
| [予定]  | Stage 3: OpenCC 相当の文脈変換（`chinese` feature） |

詳細は [`tasks/todo.md`](tasks/todo.md) を参照してください。

---

## ビルド & テスト

```bash
cargo build
cargo test
cargo test --features nfkc     # NFKC 正規化を含む
cargo test --features index    # FST コーパスインデックスを含む
cargo clippy --all-targets -- -D warnings
cargo fmt
cargo bench                    # criterion が必要
```

---

## 名前の由来

**解字（かいじ）** — 「文字を解析し、その本来の形に還元する」こと。

後漢の許慎が著した『説文解字』（約 100 年）に由来する概念です。
同書の各項目は、ある表記形から出発し、その文字の正統な構造・意味を導き出します。
kaiji ライブラリがやっていることと寸分違わず同じです。

---

## ライセンス

MIT OR Apache-2.0
