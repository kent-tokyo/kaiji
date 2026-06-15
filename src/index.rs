//! FST-backed corpus index for approximate CJK string search.
//!
//! Enable with the `index` Cargo feature.
//!
//! # Example
//!
//! ```rust
//! use kaiji::index::KaijiIndex;
//! use kaiji::NormalizerConfig;
//!
//! let corpus = vec![
//!     "斎藤一郎".to_string(),
//!     "渡辺花子".to_string(),
//!     "佐藤次郎".to_string(),
//! ];
//! let index = KaijiIndex::build(corpus, NormalizerConfig::default()).unwrap();
//!
//! // Variant-form query still finds the canonical entry
//! let hits = index.search("齋藤一郎", 0.9).unwrap();
//! assert!(!hits.is_empty());
//! assert_eq!(hits[0].original, "斎藤一郎");
//! assert_eq!(hits[0].score, 1.0);
//! ```

use std::collections::BTreeMap;

use fst::{Map, MapBuilder, Streamer};

use crate::config::NormalizerConfig;
use crate::error::{CjkFuzzyError, Result};
use crate::normalizer::Normalizer;
use crate::similarity;

/// A single result from [`KaijiIndex::search`].
pub struct SearchHit {
    /// The original string from the corpus (before normalization).
    pub original: String,
    /// Jaro-Winkler similarity between the normalized query and the normalized corpus key,
    /// in `[0.0, 1.0]`.
    pub score: f32,
}

/// An in-memory FST-backed index for approximate CJK string search.
///
/// Build once with [`KaijiIndex::build`], then query repeatedly with [`KaijiIndex::search`].
/// Multiple originals that normalize to the same canonical form are stored in the same slot;
/// all are returned when that slot scores above the threshold.
pub struct KaijiIndex {
    /// Maps normalized form (UTF-8 bytes) → slot index.
    fst: Map<Vec<u8>>,
    /// slot index → list of original (un-normalized) strings.
    slots: Vec<Vec<String>>,
    config: NormalizerConfig,
}

impl KaijiIndex {
    /// Build an index from an iterator of strings, normalizing each with `config`.
    ///
    /// Strings that normalize to the same canonical form share a slot; all are returned
    /// together when that slot matches a query. The FST is built from a `BTreeMap` so
    /// keys are automatically inserted in sorted order.
    ///
    /// # Errors
    ///
    /// Returns [`CjkFuzzyError`] if normalization or FST construction fails.
    pub fn build<I>(iter: I, config: NormalizerConfig) -> Result<Self>
    where
        I: IntoIterator<Item = String>,
    {
        let normalizer = Normalizer::with_config(config.clone());

        // BTreeMap keeps keys in sorted order, satisfying the FST insertion invariant.
        let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for item in iter {
            let normed = normalizer.normalize(&item)?.into_owned();
            groups.entry(normed).or_default().push(item);
        }

        let mut slots: Vec<Vec<String>> = Vec::with_capacity(groups.len());
        let mut builder = MapBuilder::memory();

        for (normed, originals) in groups {
            let slot_id = slots.len() as u64;
            builder
                .insert(normed.as_bytes(), slot_id)
                .map_err(|e| CjkFuzzyError::InvalidInput(e.to_string()))?;
            slots.push(originals);
        }

        let bytes = builder
            .into_inner()
            .map_err(|e| CjkFuzzyError::InvalidInput(e.to_string()))?;
        let fst = Map::new(bytes).map_err(|e| CjkFuzzyError::InvalidInput(e.to_string()))?;

        Ok(Self { fst, slots, config })
    }

    /// Search for corpus entries similar to `query`.
    ///
    /// Normalizes `query` with the index's config, then scores every entry in the FST
    /// using Jaro-Winkler similarity on the normalized forms. Returns all entries whose
    /// score meets or exceeds `threshold`, sorted by score descending.
    ///
    /// For an exact lookup (no fuzzy scoring), pass `threshold = 1.0`.
    ///
    /// # Errors
    ///
    /// Returns [`CjkFuzzyError`] if query normalization fails.
    pub fn search(&self, query: &str, threshold: f32) -> Result<Vec<SearchHit>> {
        let normalizer = Normalizer::with_config(self.config.clone());
        let normed_query = normalizer.normalize(query)?;

        let mut hits: Vec<SearchHit> = Vec::new();
        let mut stream = self.fst.stream();

        while let Some((key_bytes, slot_id)) = stream.next() {
            let normed_key = std::str::from_utf8(key_bytes)
                .map_err(|e| CjkFuzzyError::InvalidInput(e.to_string()))?;
            let score = similarity::score_strs(&normed_query, normed_key);
            if score >= threshold {
                for original in &self.slots[slot_id as usize] {
                    hits.push(SearchHit {
                        original: original.clone(),
                        score,
                    });
                }
            }
        }

        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(hits)
    }

    /// The number of unique normalized forms in the index.
    pub fn len(&self) -> usize {
        self.slots.len()
    }

    /// Returns `true` if the index contains no entries.
    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> NormalizerConfig {
        NormalizerConfig::default()
    }

    fn build_small() -> KaijiIndex {
        let corpus = vec![
            "斎藤一郎".to_string(),
            "齋藤一郎".to_string(), // variant — same slot as 斎藤一郎
            "渡辺花子".to_string(),
            "佐藤次郎".to_string(),
            "鈴木三郎".to_string(),
        ];
        KaijiIndex::build(corpus, default_config()).unwrap()
    }

    #[test]
    fn len_deduplicates_normalized() {
        let idx = build_small();
        // 斎藤一郎 and 齋藤一郎 share a slot → 4 unique normalized forms
        assert_eq!(idx.len(), 4);
        assert!(!idx.is_empty());
    }

    #[test]
    fn exact_match_via_variant_query() {
        let idx = build_small();
        // 齋藤一郎 normalizes to same form as 斎藤一郎 → score 1.0
        let hits = idx.search("齋藤一郎", 1.0).unwrap();
        assert_eq!(hits.len(), 2, "both originals in the slot should be returned");
        assert_eq!(hits[0].score, 1.0);
        // Both originals should appear
        let originals: Vec<&str> = hits.iter().map(|h| h.original.as_str()).collect();
        assert!(originals.contains(&"斎藤一郎"));
        assert!(originals.contains(&"齋藤一郎"));
    }

    #[test]
    fn fuzzy_match_finds_partial_overlap() {
        let idx = build_small();
        // 斎藤二郎 ≈ 斎藤一郎 (differ only in 一/二)
        let hits = idx.search("斎藤二郎", 0.7).unwrap();
        assert!(!hits.is_empty());
        assert!(hits.iter().any(|h| h.original == "斎藤一郎" || h.original == "齋藤一郎"));
    }

    #[test]
    fn no_match_below_threshold() {
        let idx = build_small();
        let hits = idx.search("田中太郎", 0.99).unwrap();
        assert!(hits.is_empty());
    }

    #[test]
    fn results_sorted_descending() {
        let idx = build_small();
        let hits = idx.search("斎藤", 0.0).unwrap();
        for pair in hits.windows(2) {
            assert!(pair[0].score >= pair[1].score);
        }
    }

    #[test]
    fn empty_index() {
        let idx = KaijiIndex::build(std::iter::empty(), default_config()).unwrap();
        assert!(idx.is_empty());
        assert_eq!(idx.len(), 0);
        let hits = idx.search("斎藤", 0.5).unwrap();
        assert!(hits.is_empty());
    }

    #[test]
    fn build_large_corpus_no_panic() {
        let corpus: Vec<String> = (0..1000).map(|i| format!("名前{i:04}")).collect();
        let idx = KaijiIndex::build(corpus, default_config()).unwrap();
        assert_eq!(idx.len(), 1000);
    }
}
