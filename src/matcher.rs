use crate::config::NormalizerConfig;
use crate::error::Result;
use crate::normalize::normalize;

/// Return `true` if `a` and `b` are equivalent under `config` normalization.
pub fn matches(a: &str, b: &str, config: &NormalizerConfig) -> Result<bool> {
    let na = normalize(a, config)?;
    let nb = normalize(b, config)?;
    Ok(na == nb)
}

/// Convenience wrapper using [`NormalizerConfig::default`].
pub fn matches_default(a: &str, b: &str) -> Result<bool> {
    matches(a, b, &NormalizerConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saito_variants_match() {
        assert!(matches_default("斎藤", "齋藤").unwrap());
        assert!(matches_default("斎藤", "齊藤").unwrap());
        assert!(matches_default("斎藤", "斉藤").unwrap());
    }

    #[test]
    fn watanabe_variants_match() {
        assert!(matches_default("渡辺", "渡邊").unwrap());
        assert!(matches_default("渡辺", "渡邉").unwrap());
    }

    #[test]
    fn different_names_do_not_match() {
        assert!(!matches_default("斎藤", "佐藤").unwrap());
    }

    #[test]
    fn ivs_stripped_before_compare() {
        let with_ivs: String = ['斉', '\u{E0100}', '藤'].iter().collect();
        assert!(matches_default("斉藤", &with_ivs).unwrap());
    }

    #[test]
    fn tsuchi_yoshi_matches_kichi() {
        assert!(matches_default("𠮷野家", "吉野家").unwrap());
    }

    #[test]
    fn empty_strings_match() {
        assert!(matches_default("", "").unwrap());
    }

    #[test]
    fn empty_vs_nonempty_does_not_match() {
        assert!(!matches_default("", "斉").unwrap());
    }
}
