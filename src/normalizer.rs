use std::borrow::Cow;
#[cfg(feature = "chinese")]
use std::collections::HashMap;
#[cfg(feature = "chinese")]
use std::sync::Arc;

use crate::config::NormalizerConfig;
use crate::error::Result;

/// A configured CJK normalization pipeline.
///
/// Create via [`Normalizer::builder()`] for ergonomic defaults, or supply a
/// [`NormalizerConfig`] directly via [`Normalizer::with_config`].
pub struct Normalizer {
    config: NormalizerConfig,
    /// Additional word mappings for Stage 3 Chinese conversion.
    #[cfg(feature = "chinese")]
    chinese_extra: Option<Arc<HashMap<String, String>>>,
}

/// Builder for [`Normalizer`].
pub struct NormalizerBuilder {
    config: NormalizerConfig,
    #[cfg(feature = "chinese")]
    chinese_extra: Option<Arc<HashMap<String, String>>>,
}

impl Normalizer {
    /// Returns a builder with Japanese-oriented defaults:
    /// `strip_ivs: true`, `fold_variants: true`, `width_normalization: true`.
    ///
    /// Note: these defaults differ from [`NormalizerConfig::default()`], which
    /// keeps `width_normalization: false` for backward compatibility with the
    /// standalone `normalize()` function.
    pub fn builder() -> NormalizerBuilder {
        NormalizerBuilder {
            config: builder_config(),
            #[cfg(feature = "chinese")]
            chinese_extra: None,
        }
    }

    /// Create a `Normalizer` from an explicit [`NormalizerConfig`].
    pub fn with_config(config: NormalizerConfig) -> Self {
        Self {
            config,
            #[cfg(feature = "chinese")]
            chinese_extra: None,
        }
    }

    /// Normalize `input` according to this normalizer's configuration.
    ///
    /// Returns `Cow::Borrowed` when no substitutions are made (zero allocation).
    pub fn normalize<'a>(&self, input: &'a str) -> Result<Cow<'a, str>> {
        run_pipeline(
            input,
            &self.config,
            #[cfg(feature = "chinese")]
            self.chinese_extra.as_deref(),
        )
    }

    /// Return `true` if `a` and `b` are equivalent under this normalizer's configuration.
    pub fn matches(&self, a: &str, b: &str) -> Result<bool> {
        let na = self.normalize(a)?;
        let nb = self.normalize(b)?;
        Ok(na == nb)
    }

    /// Compute a Jaro-Winkler similarity score between `a` and `b` using this
    /// normalizer's full pipeline (including width normalization) before comparison.
    ///
    /// Returns a value in `[0.0, 1.0]` where `1.0` means identical after normalization.
    pub fn similarity(&self, a: &str, b: &str) -> Result<f32> {
        let na = self.normalize(a)?;
        let nb = self.normalize(b)?;
        Ok(crate::similarity::score_strs(&na, &nb))
    }
}

impl NormalizerBuilder {
    /// Enable or disable IVS selector stripping (Stage 2).
    pub fn strip_ivs(mut self, v: bool) -> Self {
        self.config.strip_ivs = v;
        self
    }

    /// Enable or disable CJK semantic variant folding (Stage 2), e.g. 齋 → 斉.
    pub fn fold_variants(mut self, v: bool) -> Self {
        self.config.fold_variants = v;
        self
    }

    /// Enable or disable ASCII/fullwidth/halfwidth case folding.
    pub fn case_fold(mut self, v: bool) -> Self {
        self.config.case_fold = v;
        self
    }

    /// Enable or disable Stage 1 width normalization:
    /// fullwidth ASCII → halfwidth and halfwidth katakana → fullwidth (with dakuten composition).
    pub fn width_normalization(mut self, v: bool) -> Self {
        self.config.width_normalization = v;
        self
    }

    /// Convert hiragana → katakana as part of Stage 1.
    pub fn kana_to_katakana(mut self, v: bool) -> Self {
        self.config.kana_to_katakana = v;
        self
    }

    /// Convert katakana → hiragana as part of Stage 1.
    pub fn kana_to_hiragana(mut self, v: bool) -> Self {
        self.config.kana_to_hiragana = v;
        self
    }

    /// Enable or disable Unicode NFKC normalization (Stage 1b).
    /// Has no effect unless the `nfkc` Cargo feature is enabled.
    pub fn nfkc(mut self, v: bool) -> Self {
        self.config.nfkc = v;
        self
    }

    /// Convert fullwidth katakana to halfwidth katakana (Stage 1c-post).
    /// See [`NormalizerConfig::katakana_to_halfwidth`] for details.
    pub fn katakana_to_halfwidth(mut self, v: bool) -> Self {
        self.config.katakana_to_halfwidth = v;
        self
    }

    /// Convert kana to Modified Hepburn romaji (Stage 1d).
    /// See [`NormalizerConfig::kana_to_romaji`] for full behavior and caveats.
    pub fn kana_to_romaji(mut self, v: bool) -> Self {
        self.config.kana_to_romaji = v;
        self
    }

    /// Normalize historical/obsolete kana to modern equivalents (Stage 1c-pre).
    /// See [`NormalizerConfig::normalize_historical_kana`] for details and caveats.
    pub fn normalize_historical_kana(mut self, v: bool) -> Self {
        self.config.normalize_historical_kana = v;
        self
    }

    /// Set Stage 3 Chinese Traditional↔Simplified word conversion mode.
    /// Has no effect unless the `chinese` Cargo feature is enabled.
    pub fn chinese_convert(mut self, mode: crate::config::ChineseConvertMode) -> Self {
        self.config.chinese_convert = mode;
        self
    }

    /// Supply additional word mappings for Stage 3 Chinese conversion.
    /// Built-in entries take priority over these when both match a candidate.
    /// Has no effect unless the `chinese` Cargo feature is enabled.
    #[cfg(feature = "chinese")]
    pub fn chinese_extra_words(mut self, map: HashMap<String, String>) -> Self {
        self.chinese_extra = Some(Arc::new(map));
        self
    }

    /// Consume the builder and return a configured [`Normalizer`].
    pub fn build(self) -> Normalizer {
        Normalizer {
            config: self.config,
            #[cfg(feature = "chinese")]
            chinese_extra: self.chinese_extra,
        }
    }
}

/// Run the full normalization pipeline according to `config`.
///
/// Pipeline order:
///   Stage 1a — width normalization (fullwidth↔halfwidth)
///   Stage 1b — NFKC (optional feature gate)
///   Stage 2  — IVS strip + variant folding
/// Returns the builder's Japanese-oriented defaults.
///
/// Differs from `NormalizerConfig::default()` in that `width_normalization` is
/// `true` here, matching typical builder usage. `NormalizerConfig::default()`
/// keeps it `false` to preserve backward compatibility with `normalize_default()`.
fn builder_config() -> NormalizerConfig {
    NormalizerConfig {
        strip_ivs: true,
        fold_variants: true,
        case_fold: false,
        width_normalization: true,
        kana_to_katakana: false,
        kana_to_hiragana: false,
        nfkc: false,
        address_normalization: false,
        katakana_to_halfwidth: false,
        kana_to_romaji: false,
        chinese_convert: crate::config::ChineseConvertMode::Off,
        normalize_historical_kana: false,
    }
}

fn apply_katakana_halfwidth_stage<'a>(cow: Cow<'a, str>) -> Cow<'a, str> {
    match cow {
        Cow::Borrowed(s) => crate::width::apply_katakana_halfwidth_fold(s),
        Cow::Owned(s) => {
            let result = crate::width::apply_katakana_halfwidth_fold(&s);
            match result {
                Cow::Owned(out) => Cow::Owned(out),
                Cow::Borrowed(_) => Cow::Owned(s),
            }
        }
    }
}

fn apply_romaji_stage<'a>(cow: Cow<'a, str>) -> Cow<'a, str> {
    // Romaji always changes script (kana → ASCII) → always Owned
    match cow {
        Cow::Borrowed(s) => Cow::Owned(crate::romaji::kana_to_romaji(s)),
        Cow::Owned(s)    => Cow::Owned(crate::romaji::kana_to_romaji(&s)),
    }
}

fn apply_historical_kana_stage<'a>(cow: Cow<'a, str>) -> Cow<'a, str> {
    match cow {
        Cow::Borrowed(s) => crate::kana::apply_historical_kana_fold(s),
        Cow::Owned(s) => {
            let result = crate::kana::apply_historical_kana_fold(&s);
            match result {
                Cow::Owned(out) => Cow::Owned(out),
                Cow::Borrowed(_) => Cow::Owned(s),
            }
        }
    }
}

fn apply_kana_stage<'a>(cow: Cow<'a, str>, to_kata: bool, to_hira: bool) -> Cow<'a, str> {
    match cow {
        Cow::Borrowed(s) => crate::kana::apply_kana_fold(s, to_kata, to_hira),
        Cow::Owned(s) => {
            let result = crate::kana::apply_kana_fold(&s, to_kata, to_hira);
            match result {
                Cow::Owned(out) => Cow::Owned(out),
                Cow::Borrowed(_) => Cow::Owned(s),
            }
        }
    }
}

/// Apply Stage 2 (IVS strip + variant folding) to `cow`.
///
/// Handles the lifetime split: when Stage 1 produced a `Borrowed` the `'a`
/// lifetime can propagate through Stage 2 unchanged. When Stage 1 produced an
/// `Owned` string, Stage 2 borrows it; if Stage 2 is a no-op the owned `String`
/// is reclaimed so no extra allocation occurs.
fn apply_stage2<'a>(cow: Cow<'a, str>, config: &NormalizerConfig) -> Result<Cow<'a, str>> {
    match cow {
        Cow::Borrowed(s) => crate::normalize::normalize(s, config),
        Cow::Owned(s) => {
            let after2 = crate::normalize::normalize(&s, config)?;
            match after2 {
                Cow::Owned(out) => Ok(Cow::Owned(out)),
                Cow::Borrowed(_) => Ok(Cow::Owned(s)),
            }
        }
    }
}

fn run_pipeline<'a>(
    input: &'a str,
    config: &NormalizerConfig,
    #[cfg(feature = "chinese")] chinese_extra: Option<&HashMap<String, String>>,
) -> Result<Cow<'a, str>> {
    if input.is_empty() {
        return Ok(Cow::Borrowed(input));
    }

    // Stage 1a: width normalization
    let after_width: Cow<'a, str> = if config.width_normalization {
        crate::width::convert_width(input)
    } else {
        Cow::Borrowed(input)
    };

    // Stage 1b: NFKC (only when the `nfkc` feature is compiled in)
    #[cfg(feature = "nfkc")]
    let after_width: Cow<'_, str> = if config.nfkc {
        crate::width::apply_nfkc(after_width)
    } else {
        after_width
    };

    // Stage 1c-pre: historical kana → modern equivalents
    // Must run before Stage 1c so ぢ→じ→ジ (with kana_to_katakana) chains correctly.
    let after_width = if config.normalize_historical_kana {
        apply_historical_kana_stage(after_width)
    } else {
        after_width
    };

    // Stage 1c: kana normalization (ひらがな↔カタカナ)
    let after_kana = if config.kana_to_katakana || config.kana_to_hiragana {
        apply_kana_stage(after_width, config.kana_to_katakana, config.kana_to_hiragana)
    } else {
        after_width
    };

    // Stage 1c-post: fullwidth katakana → halfwidth katakana
    let after_kana = if config.katakana_to_halfwidth {
        apply_katakana_halfwidth_stage(after_kana)
    } else {
        after_kana
    };

    // Stage 1d: kana → Modified Hepburn romaji
    let after_kana = if config.kana_to_romaji {
        apply_romaji_stage(after_kana)
    } else {
        after_kana
    };

    // Stage 2: IVS strip + variant folding
    let after_stage2 = apply_stage2(after_kana, config)?;

    // Stage 3: word-level Chinese Traditional↔Simplified conversion
    #[cfg(feature = "chinese")]
    let after_stage2 = {
        use crate::config::ChineseConvertMode;
        if config.chinese_convert != ChineseConvertMode::Off {
            match after_stage2 {
                Cow::Borrowed(s) => {
                    crate::word_convert::convert_words(s, config.chinese_convert, chinese_extra)
                }
                Cow::Owned(s) => {
                    let result = crate::word_convert::convert_words(
                        &s,
                        config.chinese_convert,
                        chinese_extra,
                    );
                    match result {
                        Cow::Owned(out) => Cow::Owned(out),
                        Cow::Borrowed(_) => Cow::Owned(s),
                    }
                }
            }
        } else {
            after_stage2
        }
    };

    // Stage 4: address normalization (Japanese kanji numerals → Arabic)
    #[cfg(feature = "address")]
    let final_result: Cow<'_, str> = if config.address_normalization {
        let s = after_stage2.into_owned();
        let normalized = crate::address::normalize_address(&s);
        if normalized == s {
            Cow::Owned(s)
        } else {
            Cow::Owned(normalized)
        }
    } else {
        after_stage2
    };

    #[cfg(not(feature = "address"))]
    let final_result = after_stage2;

    Ok(final_result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NormalizerConfig, normalize, normalize_default};

    #[test]
    fn builder_defaults_enable_width() {
        let n = Normalizer::builder().build();
        assert_eq!(n.normalize("ＡＢＣ").unwrap(), "ABC");
    }

    #[test]
    fn builder_fold_variants_on_by_default() {
        let n = Normalizer::builder().build();
        assert_eq!(n.normalize("齋藤").unwrap(), "斉藤");
    }

    #[test]
    fn builder_width_false_no_conversion() {
        let n = Normalizer::builder().width_normalization(false).build();
        let result = n.normalize("ＡＢＣ").unwrap();
        assert_eq!(result, "ＡＢＣ");
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn builder_matches_legacy_api_when_width_disabled() {
        let n = Normalizer::builder().width_normalization(false).build();
        let config = NormalizerConfig::default();
        let input = "齋藤一郎";
        assert_eq!(
            n.normalize(input).unwrap(),
            normalize(input, &config).unwrap()
        );
    }

    #[test]
    fn normalizer_matches_true() {
        let n = Normalizer::builder().build();
        assert!(n.matches("齋藤", "斉藤").unwrap());
    }

    #[test]
    fn normalizer_matches_false() {
        let n = Normalizer::builder().build();
        assert!(!n.matches("齋藤", "佐藤").unwrap());
    }

    #[test]
    fn pipeline_stage1_then_stage2() {
        let n = Normalizer::builder().build();
        assert_eq!(n.normalize("ＡＢＣ齋藤").unwrap(), "ABC斉藤");
    }

    #[test]
    fn zero_copy_when_no_change() {
        let n = Normalizer::builder().build();
        // Pure ASCII: no fullwidth, no variants
        let result = n.normalize("hello").unwrap();
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn legacy_normalize_default_unaffected_by_width() {
        // normalize_default uses NormalizerConfig::default() which has width_normalization: false
        let result = normalize_default("ＡＢＣ").unwrap();
        assert_eq!(result, "ＡＢＣ");
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn empty_string() {
        let n = Normalizer::builder().build();
        let result = n.normalize("").unwrap();
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(result, "");
    }

    #[test]
    fn with_config_uses_supplied_config() {
        let config = NormalizerConfig {
            width_normalization: true,
            ..NormalizerConfig::default()
        };
        let n = Normalizer::with_config(config);
        assert_eq!(n.normalize("ＡＢＣ").unwrap(), "ABC");
    }

    #[test]
    fn kana_to_katakana_pipeline() {
        let n = Normalizer::builder().kana_to_katakana(true).build();
        assert_eq!(n.normalize("ひらがな").unwrap(), "ヒラガナ");
    }

    #[test]
    fn kana_to_hiragana_pipeline() {
        let n = Normalizer::builder().kana_to_hiragana(true).build();
        assert_eq!(n.normalize("ラーメン").unwrap(), "らーめん");
    }

    #[test]
    fn kana_with_variant_fold() {
        // Kana + variant folding in one pass
        let n = Normalizer::builder().kana_to_katakana(true).build();
        assert_eq!(n.normalize("斎藤ひとし").unwrap(), "斉藤ヒトシ");
    }

    #[test]
    fn historical_kana_hiragana_normalized() {
        let n = Normalizer::builder().normalize_historical_kana(true).build();
        assert_eq!(n.normalize("ゐゑをぢづ").unwrap(), "いえおじず");
    }

    #[test]
    fn historical_kana_katakana_normalized() {
        let n = Normalizer::builder().normalize_historical_kana(true).build();
        assert_eq!(n.normalize("ヰヱヲヂヅ").unwrap(), "イエオジズ");
    }

    #[test]
    fn historical_kana_chain_kana_to_katakana() {
        // ぢ→じ (historical) then じ→ジ (kana_to_katakana)
        let n = Normalizer::builder()
            .normalize_historical_kana(true)
            .kana_to_katakana(true)
            .build();
        assert_eq!(n.normalize("ぢづ").unwrap(), "ジズ");
    }

    #[test]
    fn historical_kana_chain_kana_to_hiragana() {
        // ヂ→ジ (historical) then ジ→じ (kana_to_hiragana)
        let n = Normalizer::builder()
            .normalize_historical_kana(true)
            .kana_to_hiragana(true)
            .build();
        assert_eq!(n.normalize("ヂヅ").unwrap(), "じず");
    }

    #[test]
    fn historical_kana_default_off_leaves_unchanged() {
        let n = Normalizer::builder().build();
        let input = "ゐゑをぢづヰヱヲヂヅ";
        let result = n.normalize(input).unwrap();
        assert_eq!(result, input);
    }

    #[test]
    fn katakana_to_halfwidth_basic() {
        let n = Normalizer::builder().katakana_to_halfwidth(true).build();
        assert_eq!(n.normalize("アイウエオ").unwrap(), "ｱｲｳｴｵ");
    }

    #[test]
    fn katakana_to_halfwidth_voiced() {
        let n = Normalizer::builder().katakana_to_halfwidth(true).build();
        assert_eq!(n.normalize("ガギグ").unwrap(), "ｶﾞｷﾞｸﾞ");
    }

    #[test]
    fn kana_to_katakana_then_halfwidth() {
        let n = Normalizer::builder()
            .kana_to_katakana(true)
            .katakana_to_halfwidth(true)
            .build();
        assert_eq!(n.normalize("がぎぐ").unwrap(), "ｶﾞｷﾞｸﾞ");
    }

    #[test]
    fn width_normalization_and_katakana_halfwidth_round_trip() {
        let n = Normalizer::builder()
            .width_normalization(true)
            .katakana_to_halfwidth(true)
            .build();
        assert_eq!(n.normalize("ｶﾞ").unwrap(), "ｶﾞ");
    }

    #[test]
    fn romaji_katakana_pipeline() {
        let n = Normalizer::builder().kana_to_romaji(true).build();
        assert_eq!(n.normalize("サトウケンジ").unwrap(), "satokenji");
    }

    #[test]
    fn romaji_hiragana_pipeline() {
        let n = Normalizer::builder().kana_to_romaji(true).build();
        assert_eq!(n.normalize("さとうけんじ").unwrap(), "satokenji");
    }

    #[test]
    fn romaji_mixed_kanji_kana() {
        let n = Normalizer::builder().kana_to_romaji(true).build();
        assert_eq!(n.normalize("斉藤ゆうき").unwrap(), "斉藤yuki");
    }

    #[test]
    fn romaji_variant_fold_then_romaji() {
        // Stage 2 (variant fold) applies to the non-kana portion; kana→romaji via Stage 1d
        let n = Normalizer::builder().kana_to_romaji(true).build();
        assert_eq!(n.normalize("齋藤ゆうき").unwrap(), "斉藤yuki");
    }

    #[test]
    fn romaji_false_no_change() {
        let n = Normalizer::builder().kana_to_romaji(false).build();
        let result = n.normalize("アイウエオ").unwrap();
        assert_eq!(result, "アイウエオ");
    }

    #[test]
    fn historical_kana_zero_alloc_no_historical() {
        let n = Normalizer::builder()
            .normalize_historical_kana(true)
            .width_normalization(false)
            .fold_variants(false)
            .strip_ivs(false)
            .build();
        let result = n.normalize("あいうえおアイウエオ").unwrap();
        assert!(matches!(result, Cow::Borrowed(_)));
    }
}
