use wasm_bindgen::prelude::*;
use js_sys;

fn map_err(e: kaiji::CjkFuzzyError) -> JsError {
    JsError::new(&e.to_string())
}

/// Normalize a CJK string using default configuration.
///
/// Folds variant characters (e.g. 齋→斉) and strips IVS selectors.
/// Width normalization is disabled; use [`Normalizer`] for the full pipeline.
#[wasm_bindgen]
pub fn normalize(input: &str) -> Result<String, JsError> {
    kaiji::normalize_default(input)
        .map(|s| s.into_owned())
        .map_err(map_err)
}

/// Return `true` if `a` and `b` are equivalent after CJK normalization.
#[wasm_bindgen]
pub fn matches(a: &str, b: &str) -> Result<bool, JsError> {
    kaiji::matches_default(a, b).map_err(map_err)
}

/// Compute a Jaro-Winkler similarity score (0.0–1.0) between two strings
/// after CJK normalization.
///
/// Variant forms that normalize to the same canonical form return `1.0`.
#[wasm_bindgen]
pub fn similarity_score(a: &str, b: &str) -> Result<f32, JsError> {
    kaiji::similarity_score(a, b, &kaiji::NormalizerConfig::default()).map_err(map_err)
}

/// A configured CJK normalization pipeline.
///
/// ```js
/// const n = new Normalizer(true, true, true, false, false, false);
/// n.normalize("ＡＢＣ齋藤"); // "ABC斉藤"
/// n.matches("渡辺", "渡邊");  // true
/// n.similarity("斎藤", "齋藤"); // 1.0
/// ```
#[wasm_bindgen]
pub struct Normalizer {
    inner: kaiji::Normalizer,
}

#[wasm_bindgen]
impl Normalizer {
    /// Create a `Normalizer` with explicit stage flags.
    ///
    /// - `strip_ivs` — remove invisible IVS selectors (U+E0100–U+E01EF)
    /// - `fold_variants` — map old/variant CJK chars to canonical forms (e.g. 齋→斉)
    /// - `width_normalization` — fullwidth ASCII→halfwidth, halfwidth kana→fullwidth
    /// - `case_fold` — ASCII A–Z → a–z
    /// - `kana_to_katakana` — convert hiragana to katakana
    /// - `kana_to_hiragana` — convert katakana to hiragana (long vowel ー preserved)
    #[wasm_bindgen(constructor)]
    pub fn new(
        strip_ivs: bool,
        fold_variants: bool,
        width_normalization: bool,
        case_fold: bool,
        kana_to_katakana: bool,
        kana_to_hiragana: bool,
    ) -> Normalizer {
        let inner = kaiji::Normalizer::builder()
            .strip_ivs(strip_ivs)
            .fold_variants(fold_variants)
            .width_normalization(width_normalization)
            .case_fold(case_fold)
            .kana_to_katakana(kana_to_katakana)
            .kana_to_hiragana(kana_to_hiragana)
            .build();
        Normalizer { inner }
    }

    /// Normalize `input` according to this normalizer's configuration.
    pub fn normalize(&self, input: &str) -> Result<String, JsError> {
        self.inner
            .normalize(input)
            .map(|s| s.into_owned())
            .map_err(map_err)
    }

    /// Return `true` if `a` and `b` are equivalent under this normalizer.
    pub fn matches(&self, a: &str, b: &str) -> Result<bool, JsError> {
        self.inner.matches(a, b).map_err(map_err)
    }

    /// Compute a Jaro-Winkler similarity score (0.0–1.0) under this normalizer.
    pub fn similarity(&self, a: &str, b: &str) -> Result<f32, JsError> {
        self.inner.similarity(a, b).map_err(map_err)
    }
}

/// A single result from [`KaijiIndex::search`].
#[wasm_bindgen]
pub struct SearchHit {
    #[wasm_bindgen(getter_with_clone)]
    pub original: String,
    pub score: f32,
}

/// FST-backed corpus index for approximate CJK string search.
///
/// ```js
/// const idx = new KaijiIndex(["斎藤一郎", "渡辺花子", "佐藤次郎"]);
/// const hits = idx.search("齋藤一郎", 0.9);
/// // hits is a JS Array of SearchHit objects
/// // hits[0].original === "斎藤一郎", hits[0].score === 1.0
/// ```
#[wasm_bindgen]
pub struct KaijiIndex {
    inner: kaiji::KaijiIndex,
}

#[wasm_bindgen]
impl KaijiIndex {
    /// Build an index from an array of strings.
    #[wasm_bindgen(constructor)]
    pub fn new(corpus: Vec<String>) -> Result<KaijiIndex, JsError> {
        let inner = kaiji::KaijiIndex::build(corpus, kaiji::NormalizerConfig::default())
            .map_err(map_err)?;
        Ok(KaijiIndex { inner })
    }

    /// Search for corpus entries similar to `query`.
    ///
    /// Returns a JS Array of `SearchHit` objects sorted by score descending.
    pub fn search(&self, query: &str, threshold: f32) -> Result<js_sys::Array, JsError> {
        let hits = self.inner.search(query, threshold).map_err(map_err)?;
        let arr = js_sys::Array::new_with_length(hits.len() as u32);
        for (i, h) in hits.into_iter().enumerate() {
            let hit = SearchHit {
                original: h.original,
                score: h.score,
            };
            arr.set(i as u32, wasm_bindgen::JsValue::from(hit));
        }
        Ok(arr)
    }

    /// Number of unique normalized forms in the index.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns true if the index is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_folds_variants() {
        assert_eq!(normalize("齋藤").unwrap(), "斉藤");
    }

    #[test]
    fn matches_variant_forms() {
        assert!(matches("斎藤", "齋藤").unwrap());
        assert!(!matches("斎藤", "佐藤").unwrap());
    }

    #[test]
    fn similarity_score_variant_is_one() {
        assert_eq!(similarity_score("斎藤", "齋藤").unwrap(), 1.0);
    }

    #[test]
    fn normalizer_full_pipeline() {
        let n = Normalizer::new(true, true, true, false, false, false);
        assert_eq!(n.normalize("ＡＢＣ齋藤").unwrap(), "ABC斉藤");
        assert!(n.matches("渡辺", "渡邊").unwrap());
        assert_eq!(n.similarity("斎藤", "齋藤").unwrap(), 1.0);
    }

    #[test]
    fn normalizer_case_fold() {
        let n = Normalizer::new(false, false, false, true, false, false);
        assert_eq!(n.normalize("HELLO").unwrap(), "hello");
    }
}
