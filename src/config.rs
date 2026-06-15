/// Configuration for the CJK normalizer pipeline.
///
/// Use [`NormalizerConfig::default()`] or [`crate::Normalizer::builder()`] to
/// construct this. Direct struct-literal construction from outside the crate is
/// not supported (`#[non_exhaustive]`).
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizerConfig {
    /// Stage 2: Strip IVS (Ideographic Variation Sequences, U+E0100..=U+E01EF).
    pub strip_ivs: bool,
    /// Stage 2: Fold CJK semantic variant characters to a canonical form (e.g. 齋 → 斎).
    pub fold_variants: bool,
    /// Fold ASCII/half-width/full-width case differences.
    pub case_fold: bool,
    /// Stage 1: Convert fullwidth ASCII → halfwidth and halfwidth katakana → fullwidth katakana.
    /// Defaults to `false` to preserve backward compatibility with `normalize_default()`.
    pub width_normalization: bool,
    /// Convert hiragana (U+3041–U+3096) to katakana.
    pub kana_to_katakana: bool,
    /// Convert katakana (U+30A1–U+30F6) to hiragana. Long vowel ー is preserved.
    pub kana_to_hiragana: bool,
    /// Stage 1 (optional): Apply Unicode NFKC normalization after the width pass.
    /// Requires the `nfkc` Cargo feature. When the feature is absent this field is ignored.
    pub nfkc: bool,
    /// Stage 4: Normalize Japanese address notation (漢数字 → Arabic, 丁目/番/号 unification).
    /// Requires the `address` Cargo feature. When the feature is absent this field is ignored.
    pub address_normalization: bool,
}

impl Default for NormalizerConfig {
    fn default() -> Self {
        Self {
            strip_ivs: true,
            fold_variants: true,
            case_fold: false,
            width_normalization: false,
            kana_to_katakana: false,
            kana_to_hiragana: false,
            nfkc: false,
            address_normalization: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_flags() {
        let cfg = NormalizerConfig::default();
        assert!(cfg.strip_ivs);
        assert!(cfg.fold_variants);
        assert!(!cfg.case_fold);
        assert!(!cfg.width_normalization);
        assert!(!cfg.nfkc);
    }
}
