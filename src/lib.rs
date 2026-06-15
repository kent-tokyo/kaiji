//! # kaiji — CJK Fuzzy Match & Normalization Engine
//!
//! High-performance normalization and fuzzy matching for Japanese, Chinese, and Korean text.
//!
//! ## Quick start
//!
//! ```rust
//! use kaiji::{Normalizer, matches_default, normalize_default, similarity_score, NormalizerConfig};
//!
//! // Fuzzy name matching — variant characters are folded before comparison
//! assert!(matches_default("斎藤", "齋藤").unwrap());
//! assert!(matches_default("渡辺", "渡邊").unwrap());
//!
//! // Normalization — zero allocation when input is already canonical
//! let s = normalize_default("齋藤一郎").unwrap();
//! assert_eq!(s, "斉藤一郎");
//!
//! // Similarity score (Jaro-Winkler on normalized strings)
//! let cfg = NormalizerConfig::default();
//! assert_eq!(similarity_score("斎藤", "齋藤", &cfg).unwrap(), 1.0);
//!
//! // Builder API with width normalization
//! let n = Normalizer::builder()
//!     .width_normalization(true)
//!     .fold_variants(true)
//!     .strip_ivs(true)
//!     .build();
//! assert_eq!(n.normalize("ＡＢＣ齋藤").unwrap(), "ABC斉藤");
//! ```

#[cfg(feature = "address")]
pub mod address;
#[cfg(feature = "chinese")]
pub mod word_convert;
pub(crate) mod romaji;
pub mod config;
pub mod dedup;
pub mod error;
#[cfg(feature = "index")]
pub mod index;
pub mod matcher;
pub mod normalize;
pub mod normalizer;
pub mod similarity;
pub mod variants;
pub(crate) mod width;
pub(crate) mod kana;

pub use config::{ChineseConvertMode, NormalizerConfig};
pub use dedup::group_variants;
pub use error::{CjkFuzzyError, Result};
#[cfg(feature = "index")]
pub use index::{KaijiIndex, SearchHit};
pub use matcher::{matches, matches_default};
pub use normalize::{normalize, normalize_default, normalize_iter};
pub use normalizer::{Normalizer, NormalizerBuilder};
pub use similarity::similarity_score;
