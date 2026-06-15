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
}
