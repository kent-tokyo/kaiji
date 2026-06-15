use ::kaiji::{
    index::KaijiIndex, CjkFuzzyError, Normalizer as KaijiNormalizer, NormalizerConfig,
};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

fn map_err(e: CjkFuzzyError) -> PyErr {
    PyValueError::new_err(e.to_string())
}

/// Normalize a CJK string using default configuration.
///
/// Folds variant characters (e.g. 齋→斉) and strips IVS selectors.
/// Width normalization is disabled by default; use :class:`Normalizer` for the full pipeline.
///
/// Args:
///     input: The string to normalize.
///
/// Returns:
///     The normalized string.
///
/// Raises:
///     ValueError: If normalization fails (e.g. ill-formed input).
#[pyfunction]
pub fn normalize(input: &str) -> PyResult<String> {
    ::kaiji::normalize_default(input)
        .map(|s| s.into_owned())
        .map_err(map_err)
}

/// Return ``True`` if *a* and *b* are equivalent after CJK normalization.
///
/// Args:
///     a: First string.
///     b: Second string.
///
/// Returns:
///     ``True`` if the normalized forms are identical.
///
/// Raises:
///     ValueError: If normalization fails.
#[pyfunction]
pub fn matches(a: &str, b: &str) -> PyResult<bool> {
    ::kaiji::matches_default(a, b).map_err(map_err)
}

/// Compute a Jaro-Winkler similarity score (0.0–1.0) between two strings
/// after CJK normalization.
///
/// Variant forms that normalize to the same canonical form return ``1.0``.
///
/// Args:
///     a: First string.
///     b: Second string.
///
/// Returns:
///     Similarity score in ``[0.0, 1.0]``.
///
/// Raises:
///     ValueError: If normalization fails.
#[pyfunction]
pub fn similarity_score(a: &str, b: &str) -> PyResult<f32> {
    ::kaiji::similarity_score(a, b, &NormalizerConfig::default()).map_err(map_err)
}

/// A configured CJK normalization pipeline.
///
/// Example::
///
///     n = Normalizer(strip_ivs=True, fold_variants=True, width_normalization=True, case_fold=False)
///     n.normalize("ＡＢＣ齋藤")  # "ABC斉藤"
///     n.matches("渡辺", "渡邊")   # True
///     n.similarity("斎藤", "齋藤")  # 1.0
#[pyclass]
pub struct Normalizer {
    inner: KaijiNormalizer,
}

#[pymethods]
impl Normalizer {
    /// Create a ``Normalizer`` with explicit stage flags.
    ///
    /// Args:
    ///     strip_ivs: Remove invisible IVS selectors (U+E0100–U+E01EF).
    ///     fold_variants: Map old/variant CJK chars to canonical forms (e.g. 齋→斉).
    ///     width_normalization: Fullwidth ASCII→halfwidth, halfwidth kana→fullwidth.
    ///     case_fold: ASCII A–Z → a–z.
    ///     kana_to_katakana: Convert hiragana to katakana.
    ///     kana_to_hiragana: Convert katakana to hiragana (long vowel ー preserved).
    #[new]
    #[pyo3(signature = (strip_ivs=true, fold_variants=true, width_normalization=true, case_fold=false, kana_to_katakana=false, kana_to_hiragana=false))]
    pub fn new(
        strip_ivs: bool,
        fold_variants: bool,
        width_normalization: bool,
        case_fold: bool,
        kana_to_katakana: bool,
        kana_to_hiragana: bool,
    ) -> Normalizer {
        let inner = KaijiNormalizer::builder()
            .strip_ivs(strip_ivs)
            .fold_variants(fold_variants)
            .width_normalization(width_normalization)
            .case_fold(case_fold)
            .kana_to_katakana(kana_to_katakana)
            .kana_to_hiragana(kana_to_hiragana)
            .build();
        Normalizer { inner }
    }

    /// Normalize *input* according to this normalizer's configuration.
    ///
    /// Args:
    ///     input: The string to normalize.
    ///
    /// Returns:
    ///     The normalized string.
    ///
    /// Raises:
    ///     ValueError: If normalization fails.
    pub fn normalize(&self, input: &str) -> PyResult<String> {
        self.inner
            .normalize(input)
            .map(|s| s.into_owned())
            .map_err(map_err)
    }

    /// Return ``True`` if *a* and *b* are equivalent under this normalizer.
    ///
    /// Args:
    ///     a: First string.
    ///     b: Second string.
    ///
    /// Returns:
    ///     ``True`` if the normalized forms are identical.
    ///
    /// Raises:
    ///     ValueError: If normalization fails.
    pub fn matches(&self, a: &str, b: &str) -> PyResult<bool> {
        self.inner.matches(a, b).map_err(map_err)
    }

    /// Compute a Jaro-Winkler similarity score (0.0–1.0) under this normalizer.
    ///
    /// Args:
    ///     a: First string.
    ///     b: Second string.
    ///
    /// Returns:
    ///     Similarity score in ``[0.0, 1.0]``.
    ///
    /// Raises:
    ///     ValueError: If normalization fails.
    pub fn similarity(&self, a: &str, b: &str) -> PyResult<f32> {
        self.inner.similarity(a, b).map_err(map_err)
    }

    pub fn __repr__(&self) -> String {
        "Normalizer()".to_string()
    }
}

/// A single fuzzy search result from :class:`Index`.
#[pyclass]
pub struct SearchHit {
    /// The original (un-normalized) string from the corpus.
    #[pyo3(get)]
    pub original: String,
    /// Jaro-Winkler similarity score in ``[0.0, 1.0]``.
    #[pyo3(get)]
    pub score: f32,
}

/// An in-memory FST-backed index for approximate CJK string search.
///
/// Build once, then call :meth:`search` repeatedly.
#[pyclass]
pub struct Index {
    inner: KaijiIndex,
}

#[pymethods]
impl Index {
    /// Build an index from a list of strings.
    ///
    /// Args:
    ///     corpus: List of strings to index.
    #[new]
    pub fn new(corpus: Vec<String>) -> PyResult<Index> {
        let inner = KaijiIndex::build(corpus, NormalizerConfig::default()).map_err(map_err)?;
        Ok(Index { inner })
    }

    /// Search for corpus entries similar to *query*.
    ///
    /// Args:
    ///     query: The query string.
    ///     threshold: Minimum Jaro-Winkler score (0.0–1.0) to include a hit.
    pub fn search(&self, query: &str, threshold: f32) -> PyResult<Vec<SearchHit>> {
        self.inner
            .search(query, threshold)
            .map(|hits| {
                hits.into_iter()
                    .map(|h| SearchHit {
                        original: h.original,
                        score: h.score,
                    })
                    .collect()
            })
            .map_err(map_err)
    }

    pub fn __repr__(&self) -> String {
        format!("Index(len={})", self.inner.len())
    }
}

/// Normalize a batch of CJK strings using default configuration.
///
/// Args:
///     inputs: List of strings to normalize.
#[pyfunction]
pub fn normalize_batch(inputs: Vec<String>) -> PyResult<Vec<String>> {
    inputs
        .iter()
        .map(|s| {
            ::kaiji::normalize_default(s)
                .map(|c| c.into_owned())
                .map_err(map_err)
        })
        .collect()
}

/// High-performance CJK fuzzy search and text normalization engine.
///
/// Quick start::
///
///     import kaiji
///
///     kaiji.normalize("齋藤")              # "斉藤"
///     kaiji.matches("斎藤", "齋藤")        # True
///     kaiji.similarity_score("斎藤", "齋藤")  # 1.0
///
///     n = kaiji.Normalizer(width_normalization=True)
///     n.normalize("ＡＢＣ齋藤")           # "ABC斉藤"
#[pymodule]
fn kaiji(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(normalize, m)?)?;
    m.add_function(wrap_pyfunction!(matches, m)?)?;
    m.add_function(wrap_pyfunction!(similarity_score, m)?)?;
    m.add_function(wrap_pyfunction!(normalize_batch, m)?)?;
    m.add_class::<Normalizer>()?;
    m.add_class::<SearchHit>()?;
    m.add_class::<Index>()?;
    Ok(())
}
