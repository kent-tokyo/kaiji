/// Controls Stage 3: word-level Chinese Traditional↔Simplified conversion.
///
/// When the `chinese` Cargo feature is absent this field is accepted but
/// silently ignored by the normalization pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChineseConvertMode {
    /// No conversion (default). Stage 3 is skipped with zero cost.
    #[default]
    Off,
    /// Convert Simplified Chinese word forms to Traditional Chinese.
    /// Designed for clean SC input; mixed SC/TC input is not supported.
    ToTraditional,
    /// Convert Traditional Chinese word forms to Simplified Chinese.
    /// Designed for clean TC input; many TC→SC conversions are already
    /// handled by Stage 2 character-level folding — this stage covers the remainder.
    ToSimplified,
}

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
    /// Stage 1c-post: Convert fullwidth katakana to halfwidth katakana.
    /// Voiced syllables expand: ガ→ｶﾞ; semi-voiced: パ→ﾊﾟ.
    /// Five katakana with no halfwidth form (ヮ ヰ ヱ ヵ ヶ) pass through unchanged.
    /// Inverse of the halfwidth→fullwidth katakana path in `width_normalization`.
    /// **Default: `false`**
    pub katakana_to_halfwidth: bool,
    /// Stage 1d: Convert kana (hiragana and katakana) to Modified Hepburn romaji.
    ///
    /// Non-kana characters (kanji, ASCII, numbers) pass through unchanged.
    /// Long vowel sequences collapse to passport style: ou→o, uu→u, oo→o
    /// (ei and ii are NOT collapsed). Runs after Stage 1c kana conversion.
    ///
    /// **Default: `false`** — output changes script (kana → ASCII Latin),
    /// so this must be opt-in.
    pub kana_to_romaji: bool,
    /// Stage 3: Word-level Chinese Traditional↔Simplified conversion mode.
    /// Requires the `chinese` Cargo feature; silently ignored when absent.
    pub chinese_convert: ChineseConvertMode,
    /// Normalize historical/obsolete kana to their modern equivalents (Stage 1c-pre).
    ///
    /// Converts: ゐ→い, ゑ→え, を→お, ぢ→じ, づ→ず (hiragana)
    /// and:      ヰ→イ, ヱ→エ, ヲ→オ, ヂ→ジ, ヅ→ズ (katakana)
    ///
    /// **Default: `false`** — を appears in virtually every modern Japanese sentence
    /// as an object particle. Enable only for eKYC name matching, OCR output from
    /// historical documents, or old-kana (旧かなづかい) normalization.
    pub normalize_historical_kana: bool,
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
            katakana_to_halfwidth: false,
            kana_to_romaji: false,
            chinese_convert: ChineseConvertMode::Off,
            normalize_historical_kana: false,
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
