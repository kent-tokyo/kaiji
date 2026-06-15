use crate::config::NormalizerConfig;
use crate::error::Result;

/// Compute the Jaro-Winkler similarity of two character slices.
///
/// Returns a value in `[0.0, 1.0]` where `1.0` means identical.
pub(crate) fn jaro_winkler_chars(s1: &[char], s2: &[char]) -> f32 {
    let len1 = s1.len();
    let len2 = s2.len();

    if len1 == 0 && len2 == 0 {
        return 1.0;
    }
    if len1 == 0 || len2 == 0 {
        return 0.0;
    }

    let window = (len1.max(len2) / 2).saturating_sub(1);

    let mut s1_matched = vec![false; len1];
    let mut s2_matched = vec![false; len2];
    let mut match_count = 0usize;

    for (i, &c1) in s1.iter().enumerate() {
        let lo = i.saturating_sub(window);
        let hi = (i + window + 1).min(len2);
        for j in lo..hi {
            if !s2_matched[j] && c1 == s2[j] {
                s1_matched[i] = true;
                s2_matched[j] = true;
                match_count += 1;
                break;
            }
        }
    }

    if match_count == 0 {
        return 0.0;
    }

    let mut transpositions = 0usize;
    let mut k = 0usize;
    for (i, &c1) in s1.iter().enumerate() {
        if s1_matched[i] {
            while !s2_matched[k] {
                k += 1;
            }
            if c1 != s2[k] {
                transpositions += 1;
            }
            k += 1;
        }
    }

    let m = match_count as f32;
    let t = (transpositions / 2) as f32;
    let jaro = (m / len1 as f32 + m / len2 as f32 + (m - t) / m) / 3.0;

    // Jaro-Winkler prefix bonus (up to 4 chars, weight 0.1)
    let prefix = s1
        .iter()
        .zip(s2.iter())
        .take(4)
        .take_while(|(a, b)| a == b)
        .count() as f32;

    jaro + prefix * 0.1 * (1.0 - jaro)
}

/// Compute Jaro-Winkler similarity between two already-normalized `&str` values.
pub(crate) fn score_strs(a: &str, b: &str) -> f32 {
    let ca: Vec<char> = a.chars().collect();
    let cb: Vec<char> = b.chars().collect();
    jaro_winkler_chars(&ca, &cb)
}

/// Compute a similarity score between `a` and `b` using Jaro-Winkler distance
/// on the CJK-normalized forms of both strings.
///
/// Normalization is performed with `config` before comparison, so variant
/// characters (e.g. 齋 and 斎藤) collapse to the same canonical form and
/// produce a score of `1.0`.
///
/// Returns a value in `[0.0, 1.0]`:
/// - `1.0` — strings are identical after normalization.
/// - `0.0` — strings share no common characters.
///
/// # Errors
///
/// Returns [`crate::CjkFuzzyError`] if normalization fails (e.g. ill-formed input).
///
/// # Example
///
/// ```rust
/// use kaiji::{similarity_score, NormalizerConfig};
///
/// let cfg = NormalizerConfig::default();
/// // Variant forms collapse to the same canonical string → score 1.0
/// assert_eq!(similarity_score("斎藤", "齋藤", &cfg).unwrap(), 1.0);
/// // Completely different names → low score
/// assert!(similarity_score("斎藤", "佐藤", &cfg).unwrap() < 0.9);
/// ```
pub fn similarity_score(a: &str, b: &str, config: &NormalizerConfig) -> Result<f32> {
    let na = crate::normalize::normalize(a, config)?;
    let nb = crate::normalize::normalize(b, config)?;
    Ok(score_strs(&na, &nb))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> NormalizerConfig {
        NormalizerConfig::default()
    }

    #[test]
    fn identical_strings_score_one() {
        assert_eq!(similarity_score("斉藤", "斉藤", &cfg()).unwrap(), 1.0);
    }

    #[test]
    fn variant_forms_score_one() {
        // 斎藤 and 齋藤 both normalize to 斉藤
        assert_eq!(similarity_score("斎藤", "齋藤", &cfg()).unwrap(), 1.0);
    }

    #[test]
    fn watanabe_variants_score_one() {
        assert_eq!(similarity_score("渡辺", "渡邊", &cfg()).unwrap(), 1.0);
    }

    #[test]
    fn different_names_score_low() {
        let score = similarity_score("斎藤", "佐藤", &cfg()).unwrap();
        assert!(score < 0.9, "expected < 0.9, got {score}");
    }

    #[test]
    fn empty_vs_empty_score_one() {
        assert_eq!(similarity_score("", "", &cfg()).unwrap(), 1.0);
    }

    #[test]
    fn empty_vs_nonempty_score_zero() {
        assert_eq!(similarity_score("", "斉", &cfg()).unwrap(), 0.0);
    }

    #[test]
    fn completely_different_score_zero() {
        // No shared characters
        // 田 and 王 share no characters within matching window
        let score = similarity_score("田中", "鈴木", &cfg()).unwrap();
        assert_eq!(score, 0.0);
    }

    #[test]
    fn score_in_range() {
        let score = similarity_score("斎藤一郎", "斉藤二郎", &cfg()).unwrap();
        assert!((0.0..=1.0).contains(&score), "score {score} out of range");
    }
}
