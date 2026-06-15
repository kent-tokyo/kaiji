use std::borrow::Cow;

// Halfwidth katakana U+FF65–U+FF9F → fullwidth katakana U+30FB/U+30FC/U+30A1–U+30F6
// Indexed by (codepoint - 0xFF65). Entry 0 = U+FF65, etc.
#[rustfmt::skip]
const HALF_KANA_TO_FULL: [char; 60] = [
    // FF65  FF66  FF67  FF68  FF69  FF6A  FF6B  FF6C
    '・', 'ヲ',  'ァ',  'ィ',  'ゥ',  'ェ',  'ォ',  'ャ',
    // FF6D  FF6E  FF6F  FF70  FF71  FF72  FF73  FF74
    'ュ',  'ョ',  'ッ',  'ー',  'ア',  'イ',  'ウ',  'エ',
    // FF75  FF76  FF77  FF78  FF79  FF7A  FF7B  FF7C
    'オ',  'カ',  'キ',  'ク',  'ケ',  'コ',  'サ',  'シ',
    // FF7D  FF7E  FF7F  FF80  FF81  FF82  FF83  FF84
    'ス',  'セ',  'ソ',  'タ',  'チ',  'ツ',  'テ',  'ト',
    // FF85  FF86  FF87  FF88  FF89  FF8A  FF8B  FF8C
    'ナ',  'ニ',  'ヌ',  'ネ',  'ノ',  'ハ',  'ヒ',  'フ',
    // FF8D  FF8E  FF8F  FF90  FF91  FF92  FF93  FF94
    'ヘ',  'ホ',  'マ',  'ミ',  'ム',  'メ',  'モ',  'ヤ',
    // FF95  FF96  FF97  FF98  FF99  FF9A  FF9B  FF9C
    'ユ',  'ヨ',  'ラ',  'リ',  'ル',  'レ',  'ロ',  'ワ',
    // FF9D  FF9E  FF9F
    'ン',  '゛',  '゜',  // FF9E/FF9F standalone → fullwidth dakuten/handakuten
    // pad to 64 entries
    '\0',
];

// (halfwidth base, halfwidth combining mark) → fullwidth pre-composed
// Half-kana that can receive a dakuten (ﾞ U+FF9E)
#[rustfmt::skip]
const DAKUTEN: &[(char, char)] = &[
    ('ｳ', 'ヴ'), ('ｶ', 'ガ'), ('ｷ', 'ギ'), ('ｸ', 'グ'), ('ｹ', 'ゲ'),
    ('ｺ', 'ゴ'), ('ｻ', 'ザ'), ('ｼ', 'ジ'), ('ｽ', 'ズ'), ('ｾ', 'ゼ'),
    ('ｿ', 'ゾ'), ('ﾀ', 'ダ'), ('ﾁ', 'ヂ'), ('ﾂ', 'ヅ'), ('ﾃ', 'デ'),
    ('ﾄ', 'ド'), ('ﾊ', 'バ'), ('ﾋ', 'ビ'), ('ﾌ', 'ブ'), ('ﾍ', 'ベ'),
    ('ﾎ', 'ボ'),
];

// Half-kana that can receive a handakuten (ﾟ U+FF9F)
#[rustfmt::skip]
const HANDAKUTEN: &[(char, char)] = &[
    ('ﾊ', 'パ'), ('ﾋ', 'ピ'), ('ﾌ', 'プ'), ('ﾍ', 'ペ'), ('ﾎ', 'ポ'),
];

fn dakuten_compose(base: char, mark: char) -> Option<char> {
    let table: &[(char, char)] = match mark {
        '\u{FF9E}' => DAKUTEN,
        '\u{FF9F}' => HANDAKUTEN,
        _ => return None,
    };
    table
        .iter()
        .find(|(b, _)| *b == base)
        .map(|(_, composed)| *composed)
}

fn halfwidth_kana_to_full(c: char) -> Option<char> {
    let n = c as u32;
    if (0xFF65..=0xFF9F).contains(&n) {
        let idx = (n - 0xFF65) as usize;
        let mapped = HALF_KANA_TO_FULL[idx];
        if mapped == '\0' { None } else { Some(mapped) }
    } else {
        None
    }
}

fn is_dakuten_or_handakuten(c: char) -> bool {
    c == '\u{FF9E}' || c == '\u{FF9F}'
}

/// Convert fullwidth ASCII (U+FF01–U+FF5E, U+3000) → halfwidth ASCII, and
/// halfwidth katakana (U+FF65–U+FF9F) → fullwidth katakana with dakuten/handakuten
/// look-ahead composition.
///
/// Returns `Cow::Borrowed` when no conversion is needed (zero allocation).
pub(crate) fn convert_width(input: &str) -> Cow<'_, str> {
    let mut owned: Option<String> = None;
    let mut chars = input.char_indices().peekable();

    while let Some((idx, ch)) = chars.next() {
        if let Some(base_full) = halfwidth_kana_to_full(ch) {
            // Peek at the next char to check for a combining dakuten/handakuten
            let out_char = if let Some(&(_, next_ch)) = chars.peek() {
                if is_dakuten_or_handakuten(next_ch) {
                    if let Some(composed) = dakuten_compose(ch, next_ch) {
                        chars.next(); // consume the combining mark
                        composed
                    } else {
                        base_full
                    }
                } else {
                    base_full
                }
            } else {
                base_full
            };

            if owned.is_none() {
                owned = Some(input[..idx].to_owned());
            }
            owned.as_mut().unwrap().push(out_char);
            continue;
        }

        if let Some(out_char) = convert_char(ch) {
            if owned.is_none() {
                owned = Some(input[..idx].to_owned());
            }
            owned.as_mut().unwrap().push(out_char);
        } else if let Some(ref mut s) = owned {
            s.push(ch);
        }
    }

    match owned {
        Some(s) => Cow::Owned(s),
        None => Cow::Borrowed(input),
    }
}

fn convert_char(c: char) -> Option<char> {
    let n = c as u32;
    // Fullwidth ASCII: U+FF01–U+FF5E → U+0021–U+007E (offset 0xFEE0)
    if (0xFF01..=0xFF5E).contains(&n) {
        return char::from_u32(n - 0xFEE0);
    }
    // Fullwidth space U+3000 → U+0020
    if c == '\u{3000}' {
        return Some(' ');
    }
    None
}

/// Apply Unicode NFKC normalization.
/// Only compiled when the `nfkc` Cargo feature is enabled.
#[cfg(feature = "nfkc")]
pub(crate) fn apply_nfkc(input: Cow<'_, str>) -> Cow<'_, str> {
    use unicode_normalization::UnicodeNormalization;
    let s: &str = &input;
    let normalized: String = s.nfkc().collect();
    if normalized == s {
        input
    } else {
        Cow::Owned(normalized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fullwidth_ascii_converts_to_half() {
        let result = convert_width("ＡＢＣ１２３");
        assert_eq!(result, "ABC123");
        assert!(matches!(result, Cow::Owned(_)));
    }

    #[test]
    fn fullwidth_space_converts() {
        let result = convert_width("　");
        assert_eq!(result, " ");
    }

    #[test]
    fn halfwidth_kana_base_converts() {
        let result = convert_width("ｱｲｳｴｵ");
        assert_eq!(result, "アイウエオ");
    }

    #[test]
    fn halfwidth_kana_with_dakuten_composed() {
        // ｶ + ﾞ → ガ (2 codepoints → 1)
        let input = "ｶﾞｷﾞｸﾞ";
        let result = convert_width(input);
        assert_eq!(result, "ガギグ");
        assert_eq!(result.chars().count(), 3);
    }

    #[test]
    fn halfwidth_kana_handakuten_composed() {
        let input = "ﾊﾟﾋﾟﾌﾟ";
        let result = convert_width(input);
        assert_eq!(result, "パピプ");
        assert_eq!(result.chars().count(), 3);
    }

    #[test]
    fn halfwidth_dakuten_standalone_maps_to_fullwidth() {
        // Standalone ﾞ (no preceding base kana) → ゛ (U+309B)
        let result = convert_width("ﾞ");
        assert_eq!(result, "゛");
    }

    #[test]
    fn no_change_returns_borrowed() {
        let result = convert_width("斉藤一郎");
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(result, "斉藤一郎");
    }

    #[test]
    fn plain_ascii_returns_borrowed() {
        let result = convert_width("hello world");
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn mixed_fullwidth_and_cjk() {
        let result = convert_width("ＡBC斉");
        assert_eq!(result, "ABC斉");
        assert!(matches!(result, Cow::Owned(_)));
    }

    #[test]
    fn fullwidth_tilde_and_bracket() {
        // U+FF5E FULLWIDTH TILDE → U+007E TILDE
        let result = convert_width("～");
        assert_eq!(result, "~");
    }

    #[cfg(feature = "nfkc")]
    #[test]
    fn nfkc_converts_enclosed_cjk() {
        let result = apply_nfkc(Cow::Borrowed("㈱"));
        assert_eq!(result, "(株)");
    }

    #[cfg(feature = "nfkc")]
    #[test]
    fn nfkc_no_change_when_already_normalized() {
        let result = apply_nfkc(Cow::Borrowed("斉藤"));
        assert_eq!(result, "斉藤");
    }
}
