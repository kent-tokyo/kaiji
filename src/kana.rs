use std::borrow::Cow;

pub(crate) fn hiragana_to_katakana_char(c: char) -> char {
    let cp = c as u32;
    if (0x3041..=0x3096).contains(&cp) {
        char::from_u32(cp + 0x60).unwrap_or(c)
    } else {
        c
    }
}

pub(crate) fn katakana_to_hiragana_char(c: char) -> char {
    let cp = c as u32;
    if (0x30A1..=0x30F6).contains(&cp) {
        // U+30FC ー is outside this range and stays as-is
        char::from_u32(cp - 0x60).unwrap_or(c)
    } else {
        c
    }
}

/// Convert kana characters. Returns Borrowed if no change (zero allocation).
/// `to_katakana` takes precedence if both are true.
pub(crate) fn apply_kana_fold<'a>(
    input: &'a str,
    to_katakana: bool,
    to_hiragana: bool,
) -> Cow<'a, str> {
    if !to_katakana && !to_hiragana {
        return Cow::Borrowed(input);
    }

    let mut buf = String::new();
    let mut changed = false;

    for (i, c) in input.char_indices() {
        let out_char = if to_katakana {
            hiragana_to_katakana_char(c)
        } else {
            katakana_to_hiragana_char(c)
        };

        if out_char == c {
            if changed {
                buf.push(c);
            }
        } else {
            if !changed {
                buf.reserve(input.len());
                buf.push_str(&input[..i]);
                changed = true;
            }
            buf.push(out_char);
        }
    }

    if changed { Cow::Owned(buf) } else { Cow::Borrowed(input) }
}

/// Return the modern equivalent of a historical/obsolete kana character, or `None`.
///
/// Covers the yotsugana (四つ仮名) mergers and the three obsolete kana
/// (ゐ/ヰ wi, ゑ/ヱ we, を/ヲ wo) that have phonetically merged with modern forms.
pub(crate) fn historical_kana_to_modern(c: char) -> Option<char> {
    match c {
        'ゐ' => Some('い'), // U+3090 → U+3044  wi → i
        'ゑ' => Some('え'), // U+3091 → U+3048  we → e
        'を' => Some('お'), // U+3092 → U+304A  wo → o
        'ぢ' => Some('じ'), // U+3062 → U+3058  di → ji (yotsugana)
        'づ' => Some('ず'), // U+3065 → U+305A  du → zu (yotsugana)
        'ヰ' => Some('イ'), // U+30F0 → U+30A4  wi → i
        'ヱ' => Some('エ'), // U+30F1 → U+30A8  we → e
        'ヲ' => Some('オ'), // U+30F2 → U+30AA  wo → o
        'ヂ' => Some('ジ'), // U+30C2 → U+30B8  di → ji (yotsugana)
        'ヅ' => Some('ズ'), // U+30C5 → U+30BA  du → zu (yotsugana)
        _ => None,
    }
}

/// Normalize historical/obsolete kana to modern equivalents.
/// Returns `Cow::Borrowed` with zero allocation when no historical kana are present.
pub(crate) fn apply_historical_kana_fold(input: &str) -> Cow<'_, str> {
    let mut buf = String::new();
    let mut changed = false;

    for (i, c) in input.char_indices() {
        match historical_kana_to_modern(c) {
            Some(out) => {
                if !changed {
                    buf.reserve(input.len());
                    buf.push_str(&input[..i]);
                    changed = true;
                }
                buf.push(out);
            }
            None => {
                if changed {
                    buf.push(c);
                }
            }
        }
    }

    if changed { Cow::Owned(buf) } else { Cow::Borrowed(input) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hiragana_to_katakana_basic() {
        assert_eq!(apply_kana_fold("ひらがな", true, false), "ヒラガナ");
    }

    #[test]
    fn katakana_to_hiragana_basic() {
        assert_eq!(apply_kana_fold("カタカナ", false, true), "かたかな");
    }

    #[test]
    fn long_vowel_preserved() {
        // ー (U+30FC) is outside U+30A1-U+30F6, stays as katakana
        assert_eq!(apply_kana_fold("ラーメン", false, true), "らーめん");
    }

    #[test]
    fn vu_conversion() {
        // ヴ (U+30F4) → ゔ (U+3094)
        assert_eq!(apply_kana_fold("ヴ", false, true), "ゔ");
    }

    #[test]
    fn mixed_string_kana_only() {
        // Only kana converted, other chars left unchanged
        assert_eq!(apply_kana_fold("ABC齋藤ひらがな", true, false), "ABC齋藤ヒラガナ");
    }

    #[test]
    fn no_change_returns_borrowed() {
        let result = apply_kana_fold("ABC123", true, false);
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn already_katakana_no_change() {
        let result = apply_kana_fold("カタカナ", true, false);
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn small_kana_converted() {
        // Small hiragana ぁ (U+3041) → small katakana ァ (U+30A1)
        assert_eq!(apply_kana_fold("ぁぃぅぇぉ", true, false), "ァィゥェォ");
    }

    #[test]
    fn both_false_returns_borrowed() {
        let result = apply_kana_fold("ひらがな", false, false);
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn historical_kana_hiragana_mappings() {
        assert_eq!(historical_kana_to_modern('ゐ'), Some('い'));
        assert_eq!(historical_kana_to_modern('ゑ'), Some('え'));
        assert_eq!(historical_kana_to_modern('を'), Some('お'));
        assert_eq!(historical_kana_to_modern('ぢ'), Some('じ'));
        assert_eq!(historical_kana_to_modern('づ'), Some('ず'));
    }

    #[test]
    fn historical_kana_katakana_mappings() {
        assert_eq!(historical_kana_to_modern('ヰ'), Some('イ'));
        assert_eq!(historical_kana_to_modern('ヱ'), Some('エ'));
        assert_eq!(historical_kana_to_modern('ヲ'), Some('オ'));
        assert_eq!(historical_kana_to_modern('ヂ'), Some('ジ'));
        assert_eq!(historical_kana_to_modern('ヅ'), Some('ズ'));
    }

    #[test]
    fn historical_kana_modern_chars_return_none() {
        assert_eq!(historical_kana_to_modern('あ'), None);
        assert_eq!(historical_kana_to_modern('ア'), None);
        assert_eq!(historical_kana_to_modern('A'), None);
        assert_eq!(historical_kana_to_modern('漢'), None);
    }

    #[test]
    fn apply_historical_kana_no_historical_returns_borrowed() {
        let result = apply_historical_kana_fold("あいうえお");
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn apply_historical_kana_yotsugana_hiragana() {
        assert_eq!(apply_historical_kana_fold("ぢづ"), "じず");
    }

    #[test]
    fn apply_historical_kana_yotsugana_katakana() {
        assert_eq!(apply_historical_kana_fold("ヂヅ"), "ジズ");
    }

    #[test]
    fn apply_historical_kana_obsolete_hiragana() {
        assert_eq!(apply_historical_kana_fold("ゐゑを"), "いえお");
    }

    #[test]
    fn apply_historical_kana_obsolete_katakana() {
        assert_eq!(apply_historical_kana_fold("ヰヱヲ"), "イエオ");
    }
}
