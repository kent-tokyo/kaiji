use std::borrow::Cow;

use crate::config::NormalizerConfig;
use crate::error::Result;
use crate::variants::variant_map;

// IVS variation selectors: U+E0100..=U+E01EF
fn is_ivs_selector(c: char) -> bool {
    ('\u{E0100}'..='\u{E01EF}').contains(&c)
}

/// Normalize a CJK string according to `config`.
///
/// Returns a `Cow::Borrowed` when no substitutions are made (zero allocation).
pub fn normalize<'a>(input: &'a str, config: &NormalizerConfig) -> Result<Cow<'a, str>> {
    if input.is_empty() {
        return Ok(Cow::Borrowed(input));
    }

    let map = variant_map();
    let mut owned: Option<String> = None;

    for (byte_pos, ch) in input.char_indices() {
        if config.strip_ivs && is_ivs_selector(ch) {
            if owned.is_none() {
                owned = Some(input[..byte_pos].to_owned());
            }
            continue;
        }

        let canonical = if config.fold_variants {
            map.get(&ch).copied().unwrap_or(ch)
        } else {
            ch
        };

        let canonical = if config.case_fold && canonical.is_ascii_uppercase() {
            canonical.to_ascii_lowercase()
        } else {
            canonical
        };

        if canonical != ch && owned.is_none() {
            owned = Some(input[..byte_pos].to_owned());
        }

        if let Some(ref mut s) = owned {
            s.push(canonical);
        }
    }

    match owned {
        Some(s) => Ok(Cow::Owned(s)),
        None => Ok(Cow::Borrowed(input)),
    }
}

/// Normalize `input` with default configuration.
pub fn normalize_default(input: &str) -> Result<Cow<'_, str>> {
    normalize(input, &NormalizerConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> NormalizerConfig {
        NormalizerConfig::default()
    }

    #[test]
    fn no_change_returns_borrowed() {
        let result = normalize("斉藤", &cfg()).unwrap();
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(result, "斉藤");
    }

    #[test]
    fn folds_saito_variants() {
        assert_eq!(normalize("齋藤", &cfg()).unwrap(), "斉藤");
        assert_eq!(normalize("齊藤", &cfg()).unwrap(), "斉藤");
        assert_eq!(normalize("斎藤", &cfg()).unwrap(), "斉藤");
    }

    #[test]
    #[cfg(not(feature = "chinese"))]
    fn folds_watanabe_variants() {
        assert_eq!(normalize("渡邊", &cfg()).unwrap(), "渡辺");
        assert_eq!(normalize("渡邉", &cfg()).unwrap(), "渡辺");
    }

    #[test]
    #[cfg(feature = "chinese")]
    fn folds_watanabe_variants_chinese() {
        assert_eq!(normalize("渡邊", &cfg()).unwrap(), "渡边");
        assert_eq!(normalize("渡邉", &cfg()).unwrap(), "渡边");
        assert_eq!(normalize("渡辺", &cfg()).unwrap(), "渡边"); // J-canonical also folds
    }

    #[test]
    fn folds_sawa_variants() {
        assert_eq!(normalize("澤", &cfg()).unwrap(), "沢");
    }

    #[test]
    fn strips_ivs_selector() {
        let with_ivs: String = ['斉', '\u{E0100}'].iter().collect();
        let result = normalize(&with_ivs, &cfg()).unwrap();
        assert_eq!(result, "斉");
    }

    #[test]
    fn no_fold_when_disabled() {
        let mut cfg = cfg();
        cfg.fold_variants = false;
        let result = normalize("齋藤", &cfg).unwrap();
        assert_eq!(result, "齋藤");
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn no_strip_ivs_when_disabled() {
        let with_ivs: String = ['斉', '\u{E0100}'].iter().collect();
        let mut cfg = cfg();
        cfg.strip_ivs = false;
        let result = normalize(&with_ivs, &cfg).unwrap();
        assert_eq!(result.chars().count(), 2);
    }

    #[test]
    fn tsuchi_yoshi_folded() {
        assert_eq!(normalize("𠮷野家", &cfg()).unwrap(), "吉野家");
    }

    #[test]
    #[cfg(not(feature = "chinese"))]
    fn folds_jis_extended_variants() {
        assert_eq!(normalize("廣島", &cfg()).unwrap(), "広島");
        assert_eq!(normalize("關西", &cfg()).unwrap(), "関西");
        assert_eq!(normalize("發展", &cfg()).unwrap(), "発展");
        assert_eq!(normalize("讀書", &cfg()).unwrap(), "読書");
    }

    #[test]
    #[cfg(feature = "chinese")]
    fn folds_jis_extended_variants_chinese() {
        assert_eq!(normalize("廣島", &cfg()).unwrap(), "广島"); // 廣→广, 島 unchanged
        assert_eq!(normalize("關西", &cfg()).unwrap(), "关西"); // 關→关
        assert_eq!(normalize("發展", &cfg()).unwrap(), "发展"); // 發→发
        assert_eq!(normalize("讀書", &cfg()).unwrap(), "读书"); // 讀→读, 書→书
    }

    #[test]
    fn case_fold_ascii_uppercase() {
        let cfg = NormalizerConfig {
            case_fold: true,
            ..NormalizerConfig::default()
        };
        assert_eq!(normalize("ABC", &cfg).unwrap(), "abc");
        assert_eq!(normalize("Hello", &cfg).unwrap(), "hello");
    }

    #[test]
    fn case_fold_default_off() {
        // Default config must NOT fold case
        let result = normalize("ABC", &cfg()).unwrap();
        assert_eq!(result, "ABC");
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn case_fold_preserves_cjk() {
        let cfg = NormalizerConfig {
            case_fold: true,
            ..NormalizerConfig::default()
        };
        // CJK characters are unaffected; ASCII uppercase is folded
        assert_eq!(normalize("ABC斉藤", &cfg).unwrap(), "abc斉藤");
    }
}
