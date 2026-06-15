use std::borrow::Cow;

use crate::config::NormalizerConfig;
use crate::error::Result;

/// A configured CJK normalization pipeline.
///
/// Create via [`Normalizer::builder()`] for ergonomic defaults, or supply a
/// [`NormalizerConfig`] directly via [`Normalizer::with_config`].
pub struct Normalizer {
    config: NormalizerConfig,
}

/// Builder for [`Normalizer`].
pub struct NormalizerBuilder {
    config: NormalizerConfig,
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
        }
    }

    /// Create a `Normalizer` from an explicit [`NormalizerConfig`].
    pub fn with_config(config: NormalizerConfig) -> Self {
        Self { config }
    }

    /// Normalize `input` according to this normalizer's configuration.
    ///
    /// Returns `Cow::Borrowed` when no substitutions are made (zero allocation).
    pub fn normalize<'a>(&self, input: &'a str) -> Result<Cow<'a, str>> {
        run_pipeline(input, &self.config)
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

    /// Consume the builder and return a configured [`Normalizer`].
    pub fn build(self) -> Normalizer {
        Normalizer {
            config: self.config,
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

fn run_pipeline<'a>(input: &'a str, config: &NormalizerConfig) -> Result<Cow<'a, str>> {
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

    // Stage 1c: kana normalization (ひらがな↔カタカナ)
    let after_kana = if config.kana_to_katakana || config.kana_to_hiragana {
        apply_kana_stage(after_width, config.kana_to_katakana, config.kana_to_hiragana)
    } else {
        after_width
    };

    // Stage 2: IVS strip + variant folding
    let after_stage2 = apply_stage2(after_kana, config)?;

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
}
