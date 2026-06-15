//! Stage 4: Japanese address normalization.
//!
//! Converts kanji numeral sequences that precede address unit words into
//! Arabic numerals. Characters not followed by an address unit are left
//! unchanged to avoid false-positives in names.

/// Address unit markers that trigger kanji-to-Arabic conversion.
/// Multi-character units must appear before single-character ones when they
/// share a prefix (e.g., "丁目" before "丁") so that the longest match wins.
const ADDRESS_UNITS: &[&str] = &[
    "丁目", // chome
    "番地", // banchi
    "番",   // ban
    "号",   // go
    "丁",   // cho (short form)
    "ノ",   // no (katakana)
    "の",   // no (hiragana)
];

/// Characters that are valid kanji numerals.
const KANJI_DIGITS: &[char] = &['一', '二', '三', '四', '五', '六', '七', '八', '九'];
const KANJI_UNITS: &[char] = &['十', '百', '千'];

/// Return true if `ch` can participate in a kanji numeral sequence.
#[inline]
fn is_kanji_numeral(ch: char) -> bool {
    KANJI_DIGITS.contains(&ch) || KANJI_UNITS.contains(&ch)
}

/// Parse a kanji numeral string into its Arabic value.
///
/// Handles:
/// - Basic digits: 一=1 … 九=9
/// - Position multipliers: 十=10, 百=100, 千=1000
/// - Compound numbers: 二十三=23, 三百二十五=325, 十五=15
///
/// Returns `None` if the string is empty or contains non-kanji-numeral chars.
fn kanji_to_arabic(s: &str) -> Option<u32> {
    if s.is_empty() {
        return None;
    }

    let mut result: u32 = 0;
    let mut current: u32 = 0; // accumulator before next multiplier

    for ch in s.chars() {
        match ch {
            '一' => current += 1,
            '二' => current += 2,
            '三' => current += 3,
            '四' => current += 4,
            '五' => current += 5,
            '六' => current += 6,
            '七' => current += 7,
            '八' => current += 8,
            '九' => current += 9,
            '十' => {
                // 十 alone at start means 1×10; with preceding digit means digit×10
                let multiplier = if current == 0 { 1 } else { current };
                result += multiplier * 10;
                current = 0;
            }
            '百' => {
                let multiplier = if current == 0 { 1 } else { current };
                result += multiplier * 100;
                current = 0;
            }
            '千' => {
                let multiplier = if current == 0 { 1 } else { current };
                result += multiplier * 1000;
                current = 0;
            }
            _ => return None, // not a kanji numeral char
        }
    }

    result += current; // add any trailing digit (e.g., 二十三 → result=20, current=3)

    if result == 0 {
        None
    } else {
        Some(result)
    }
}

/// Return the address unit that starts at `pos` in `s`, if any.
///
/// For `の`/`ノ`, the unit only triggers conversion when the character that
/// follows is a digit or kanji numeral (to avoid converting "の" in normal text).
fn address_unit_at(s: &str, pos: usize) -> Option<&str> {
    let tail = &s[pos..];

    for &unit in ADDRESS_UNITS {
        if let Some(after_unit) = tail.strip_prefix(unit) {
            // For の/ノ, require that the next character is a digit or kanji numeral.
            if unit == "の" || unit == "ノ" {
                let next_ch = after_unit.chars().next();
                let triggers = next_ch.is_some_and(|c| c.is_ascii_digit() || is_kanji_numeral(c));
                if !triggers {
                    continue;
                }
            }
            return Some(unit);
        }
    }
    None
}

/// Convert kanji numerals to Arabic numerals in an address string.
///
/// Only converts kanji numeral sequences that are directly followed by a
/// recognised address unit word (丁目, 番地, 番, 号, 丁, ノ, の).
/// All other text — including standalone kanji numbers that might be part of
/// a name — is left unchanged.
pub fn normalize_address(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let len = bytes.len();

    // We iterate by char boundary positions.
    let mut pos = 0;

    while pos < len {
        // Try to find a kanji numeral run starting here.
        // First, determine how long the kanji run is.
        let run_start = pos;
        let mut run_end = pos;

        // Collect chars one by one until we hit a non-kanji-numeral.
        let mut char_iter = input[pos..].char_indices();
        loop {
            match char_iter.next() {
                Some((offset, ch)) if is_kanji_numeral(ch) => {
                    run_end = pos + offset + ch.len_utf8();
                }
                _ => break,
            }
        }

        if run_end > run_start {
            // We have a non-empty kanji run [run_start, run_end).
            // Check if an address unit follows immediately.
            if let Some(unit) = address_unit_at(input, run_end) {
                let kanji_str = &input[run_start..run_end];
                if let Some(arabic) = kanji_to_arabic(kanji_str) {
                    // Emit the Arabic numeral instead of the kanji run.
                    // Use itoa-style manual push to avoid allocation.
                    let arabic_str = arabic.to_string();
                    output.push_str(&arabic_str);
                    // Emit the unit.
                    output.push_str(unit);
                    // Advance past both the run and the unit.
                    pos = run_end + unit.len();
                    continue;
                }
            }
            // Run not followed by unit or not parseable — emit first char literally
            // and advance by one char so we don't infinite-loop.
            let first_char = input[run_start..].chars().next().unwrap();
            output.push(first_char);
            pos = run_start + first_char.len_utf8();
        } else {
            // Not a kanji numeral char at pos — emit it as-is.
            let ch = input[pos..].chars().next().unwrap();
            output.push(ch);
            pos += ch.len_utf8();
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- kanji_to_arabic unit tests ---

    #[test]
    fn arabic_basic_digits() {
        assert_eq!(kanji_to_arabic("一"), Some(1));
        assert_eq!(kanji_to_arabic("九"), Some(9));
    }

    #[test]
    fn arabic_juu_alone() {
        assert_eq!(kanji_to_arabic("十"), Some(10));
    }

    #[test]
    fn arabic_juu_prefix() {
        assert_eq!(kanji_to_arabic("十五"), Some(15));
        assert_eq!(kanji_to_arabic("十二"), Some(12));
    }

    #[test]
    fn arabic_compound() {
        assert_eq!(kanji_to_arabic("二十三"), Some(23));
        assert_eq!(kanji_to_arabic("三百二十五"), Some(325));
    }

    #[test]
    fn arabic_hyaku() {
        assert_eq!(kanji_to_arabic("百"), Some(100));
        assert_eq!(kanji_to_arabic("百二十"), Some(120));
    }

    #[test]
    fn arabic_empty() {
        assert_eq!(kanji_to_arabic(""), None);
    }

    #[test]
    fn arabic_non_kanji() {
        assert_eq!(kanji_to_arabic("abc"), None);
    }

    // --- normalize_address tests (specified by the task) ---

    #[test]
    fn converts_simple_kanji_number() {
        assert_eq!(normalize_address("三丁目"), "3丁目");
    }

    #[test]
    fn converts_compound_kanji_number() {
        assert_eq!(normalize_address("二十三番地"), "23番地");
    }

    #[test]
    fn converts_full_address() {
        assert_eq!(normalize_address("一丁目二番三号"), "1丁目2番3号");
    }

    #[test]
    fn leaves_arabic_unchanged() {
        assert_eq!(normalize_address("3丁目4番5号"), "3丁目4番5号");
    }

    #[test]
    fn leaves_non_address_unchanged() {
        assert_eq!(normalize_address("渡辺花子"), "渡辺花子");
    }

    #[test]
    fn handles_juu_alone() {
        assert_eq!(normalize_address("十番"), "10番");
    }

    #[test]
    fn handles_mixed_input() {
        assert_eq!(normalize_address("東京都千代田区一丁目"), "東京都千代田区1丁目");
    }

    // --- additional edge-case tests ---

    #[test]
    fn converts_hyaku_nijuu_go() {
        assert_eq!(normalize_address("百二十五号"), "125号");
    }

    #[test]
    fn no_change_already_arabic() {
        assert_eq!(normalize_address("3-4-5"), "3-4-5");
    }

    #[test]
    fn mixed_arabic_and_kanji() {
        assert_eq!(normalize_address("三丁目4番5号"), "3丁目4番5号");
    }

    #[test]
    fn handles_banchi() {
        assert_eq!(normalize_address("二十三番地"), "23番地");
    }
}
