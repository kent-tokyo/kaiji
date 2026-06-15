//! Stage 1d: Kana → Modified Hepburn romaji conversion.
//!
//! Converts hiragana and katakana to passport-style Hepburn romaji.
//! Non-kana characters (kanji, ASCII, numbers) pass through unchanged.
//!
//! Rules:
//! - Long vowel collapse: ou→o, uu→u, oo→o (passport style; ei/ii not collapsed)
//! - ー: skipped (long vowel marker dropped, consistent with collapse rules)
//! - ッ: doubles the following consonant (っか→kka; っち→tchi per Hepburn)
//! - ン: "m" before b/m/p; "n'" before vowel kana; "n" elsewhere

use crate::kana::hiragana_to_katakana_char;

/// Compound kana (2-char sequences) → romaji.
/// Checked BEFORE single-char lookup so yōon (拗音) always take priority.
static COMPOUNDS: &[([char; 2], &str)] = &[
    // Standard yōon — base consonant + small ya/yu/yo
    (['キ', 'ャ'], "kya"), (['キ', 'ュ'], "kyu"), (['キ', 'ョ'], "kyo"),
    (['シ', 'ャ'], "sha"), (['シ', 'ュ'], "shu"), (['シ', 'ョ'], "sho"),
    (['チ', 'ャ'], "cha"), (['チ', 'ュ'], "chu"), (['チ', 'ョ'], "cho"),
    (['ニ', 'ャ'], "nya"), (['ニ', 'ュ'], "nyu"), (['ニ', 'ョ'], "nyo"),
    (['ヒ', 'ャ'], "hya"), (['ヒ', 'ュ'], "hyu"), (['ヒ', 'ョ'], "hyo"),
    (['ミ', 'ャ'], "mya"), (['ミ', 'ュ'], "myu"), (['ミ', 'ョ'], "myo"),
    (['リ', 'ャ'], "rya"), (['リ', 'ュ'], "ryu"), (['リ', 'ョ'], "ryo"),
    (['ギ', 'ャ'], "gya"), (['ギ', 'ュ'], "gyu"), (['ギ', 'ョ'], "gyo"),
    (['ジ', 'ャ'], "ja"),  (['ジ', 'ュ'], "ju"),  (['ジ', 'ョ'], "jo"),
    (['ビ', 'ャ'], "bya"), (['ビ', 'ュ'], "byu"), (['ビ', 'ョ'], "byo"),
    (['ピ', 'ャ'], "pya"), (['ピ', 'ュ'], "pyu"), (['ピ', 'ョ'], "pyo"),
    (['ヂ', 'ャ'], "ja"),  (['ヂ', 'ュ'], "ju"),  (['ヂ', 'ョ'], "jo"),
    // Foreign-sound compounds
    (['フ', 'ァ'], "fa"),  (['フ', 'ィ'], "fi"),  (['フ', 'ェ'], "fe"),  (['フ', 'ォ'], "fo"),
    (['シ', 'ェ'], "she"), (['チ', 'ェ'], "che"), (['ジ', 'ェ'], "je"),
    (['テ', 'ィ'], "ti"),  (['デ', 'ィ'], "di"),
    (['ウ', 'ィ'], "wi"),  (['ウ', 'ェ'], "we"),  (['ウ', 'ォ'], "wo"),
    (['ヴ', 'ァ'], "va"),  (['ヴ', 'ィ'], "vi"),  (['ヴ', 'ェ'], "ve"),  (['ヴ', 'ォ'], "vo"),
    (['ツ', 'ァ'], "tsa"), (['ツ', 'ィ'], "tsi"), (['ツ', 'ェ'], "tse"), (['ツ', 'ォ'], "tso"),
];

fn compound_romaji(a: char, b: char) -> Option<&'static str> {
    COMPOUNDS.iter().find(|(k, _)| k[0] == a && k[1] == b).map(|(_, v)| *v)
}

fn single_katakana_romaji(c: char) -> Option<&'static str> {
    match c {
        'ア' => Some("a"),   'イ' => Some("i"),   'ウ' => Some("u"),
        'エ' => Some("e"),   'オ' => Some("o"),
        'カ' => Some("ka"),  'キ' => Some("ki"),  'ク' => Some("ku"),
        'ケ' => Some("ke"),  'コ' => Some("ko"),
        'サ' => Some("sa"),  'シ' => Some("shi"), 'ス' => Some("su"),
        'セ' => Some("se"),  'ソ' => Some("so"),
        'タ' => Some("ta"),  'チ' => Some("chi"), 'ツ' => Some("tsu"),
        'テ' => Some("te"),  'ト' => Some("to"),
        'ナ' => Some("na"),  'ニ' => Some("ni"),  'ヌ' => Some("nu"),
        'ネ' => Some("ne"),  'ノ' => Some("no"),
        'ハ' => Some("ha"),  'ヒ' => Some("hi"),  'フ' => Some("fu"),
        'ヘ' => Some("he"),  'ホ' => Some("ho"),
        'マ' => Some("ma"),  'ミ' => Some("mi"),  'ム' => Some("mu"),
        'メ' => Some("me"),  'モ' => Some("mo"),
        'ヤ' => Some("ya"),  'ユ' => Some("yu"),  'ヨ' => Some("yo"),
        'ラ' => Some("ra"),  'リ' => Some("ri"),  'ル' => Some("ru"),
        'レ' => Some("re"),  'ロ' => Some("ro"),
        'ワ' => Some("wa"),  'ヲ' => Some("o"),
        'ガ' => Some("ga"),  'ギ' => Some("gi"),  'グ' => Some("gu"),
        'ゲ' => Some("ge"),  'ゴ' => Some("go"),
        'ザ' => Some("za"),  'ジ' => Some("ji"),  'ズ' => Some("zu"),
        'ゼ' => Some("ze"),  'ゾ' => Some("zo"),
        'ダ' => Some("da"),  'ヂ' => Some("ji"),  'ヅ' => Some("zu"),
        'デ' => Some("de"),  'ド' => Some("do"),
        'バ' => Some("ba"),  'ビ' => Some("bi"),  'ブ' => Some("bu"),
        'ベ' => Some("be"),  'ボ' => Some("bo"),
        'パ' => Some("pa"),  'ピ' => Some("pi"),  'プ' => Some("pu"),
        'ペ' => Some("pe"),  'ポ' => Some("po"),
        'ヴ' => Some("vu"),
        // Small kana used standalone (rare)
        'ァ' => Some("a"),  'ィ' => Some("i"),  'ゥ' => Some("u"),
        'ェ' => Some("e"),  'ォ' => Some("o"),
        'ャ' => Some("ya"), 'ュ' => Some("yu"), 'ョ' => Some("yo"),
        _ => None,
    }
}

/// ン context-sensitive romaji: "m" before b/m/p, "n'" before vowel kana, "n" elsewhere.
fn nasal_romaji(chars: &[char], next_idx: usize) -> &'static str {
    let next_k = chars.get(next_idx).map(|&c| hiragana_to_katakana_char(c));
    match next_k {
        Some(
            'バ' | 'ビ' | 'ブ' | 'ベ' | 'ボ'
            | 'パ' | 'ピ' | 'プ' | 'ペ' | 'ポ'
            | 'マ' | 'ミ' | 'ム' | 'メ' | 'モ',
        ) => "m",
        Some('ア' | 'イ' | 'ウ' | 'エ' | 'オ' | 'ァ' | 'ィ' | 'ゥ' | 'ェ' | 'ォ') => "n'",
        _ => "n",
    }
}

/// Convert kana (hiragana and/or katakana) to Modified Hepburn romaji.
///
/// Non-kana characters pass through unchanged. Long vowel sequences collapse
/// to passport style: ou→o, uu→u, oo→o (ei and ii are NOT collapsed).
pub(crate) fn kana_to_romaji(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(input.len() * 2);
    let mut i = 0usize;
    let mut pending_sokuon = false;
    let mut prev_vowel: Option<char> = None;

    while i < n {
        let ch = chars[i];
        let k = hiragana_to_katakana_char(ch);

        match k {
            'ッ' => {
                pending_sokuon = true;
                i += 1;
                continue;
            }
            'ン' => {
                out.push_str(nasal_romaji(&chars, i + 1));
                pending_sokuon = false;
                prev_vowel = None;
                i += 1;
                continue;
            }
            'ー' => {
                i += 1;
                continue;
            }
            _ => {}
        }

        // Try 2-char compound first, then single char
        let (romaji_opt, advance): (Option<&'static str>, usize) = if i + 1 < n {
            let next_k = hiragana_to_katakana_char(chars[i + 1]);
            if let Some(r) = compound_romaji(k, next_k) {
                (Some(r), 2)
            } else {
                (single_katakana_romaji(k), 1)
            }
        } else {
            (single_katakana_romaji(k), 1)
        };

        if let Some(r) = romaji_opt {
            // Modified Hepburn long-vowel collapse (ou→o, uu→u, oo→o).
            // Only exact "u" and "o" tokens collapse — compound romaji like "fu" never does.
            let collapsed = matches!(
                (r, prev_vowel),
                ("u", Some('o')) | ("u", Some('u')) | ("o", Some('o'))
            );

            if collapsed {
                pending_sokuon = false;
                // prev_vowel unchanged: long vowel still sounds like the same vowel
            } else {
                if pending_sokuon {
                    if r.starts_with("ch") {
                        // っ + ch* → "t" + romaji (Hepburn: っち→tchi, not cchi)
                        out.push('t');
                    } else {
                        let first = r.chars().next().unwrap();
                        if !"aeiou".contains(first) {
                            out.push(first);
                        }
                    }
                    pending_sokuon = false;
                }
                out.push_str(r);
                let last = r.chars().last().unwrap();
                prev_vowel = if "aeiou".contains(last) { Some(last) } else { None };
            }

            i += advance;
        } else {
            // Non-kana: pass through original character
            out.push(ch);
            pending_sokuon = false;
            prev_vowel = None;
            i += 1;
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_vowels_katakana() {
        assert_eq!(kana_to_romaji("アイウエオ"), "aiueo");
    }

    #[test]
    fn basic_vowels_hiragana() {
        assert_eq!(kana_to_romaji("あいうえお"), "aiueo");
    }

    #[test]
    fn hepburn_irregulars() {
        assert_eq!(kana_to_romaji("シチツフジズ"), "shichitsufujizu");
    }

    #[test]
    fn compound_kana_priority() {
        // シャ → "sha" (compound beats シ="shi" + ャ="ya")
        assert_eq!(kana_to_romaji("シャ"), "sha");
        assert_eq!(kana_to_romaji("チャ"), "cha");
        assert_eq!(kana_to_romaji("ジャ"), "ja");
        assert_eq!(kana_to_romaji("キャ"), "kya");
        assert_eq!(kana_to_romaji("ショ"), "sho");
    }

    #[test]
    fn compound_with_long_vowel_collapse() {
        // シャチョウ: sha + cho + u(collapse ou→o) → "shacho"
        assert_eq!(kana_to_romaji("シャチョウ"), "shacho");
    }

    #[test]
    fn sokuon_basic() {
        assert_eq!(kana_to_romaji("っか"), "kka");
        assert_eq!(kana_to_romaji("っし"), "sshi");
    }

    #[test]
    fn sokuon_before_ch() {
        // Hepburn: っち→tchi (not cchi)
        assert_eq!(kana_to_romaji("っち"), "tchi");
        assert_eq!(kana_to_romaji("っちゃ"), "tcha");
    }

    #[test]
    fn nasal_before_bilabial() {
        assert_eq!(kana_to_romaji("ンバ"), "mba");
        assert_eq!(kana_to_romaji("ンマ"), "mma");
        assert_eq!(kana_to_romaji("ンパ"), "mpa");
    }

    #[test]
    fn nasal_before_vowel() {
        assert_eq!(kana_to_romaji("ンア"), "n'a");
    }

    #[test]
    fn nasal_elsewhere() {
        assert_eq!(kana_to_romaji("ンカ"), "nka");
        assert_eq!(kana_to_romaji("ンン"), "nn");
    }

    #[test]
    fn nasal_at_end() {
        assert_eq!(kana_to_romaji("ホン"), "hon");
    }

    #[test]
    fn long_vowel_ou_collapse() {
        assert_eq!(kana_to_romaji("トウ"), "to");
        assert_eq!(kana_to_romaji("キョウ"), "kyo");
    }

    #[test]
    fn long_vowel_uu_collapse() {
        assert_eq!(kana_to_romaji("ユウ"), "yu");
    }

    #[test]
    fn long_vowel_oo_collapse() {
        assert_eq!(kana_to_romaji("オオ"), "o");
    }

    #[test]
    fn ei_not_collapsed() {
        assert_eq!(kana_to_romaji("エイ"), "ei");
    }

    #[test]
    fn ii_not_collapsed() {
        assert_eq!(kana_to_romaji("イイ"), "ii");
    }

    #[test]
    fn long_vowel_marker_skipped() {
        assert_eq!(kana_to_romaji("ラーメン"), "ramen");
    }

    #[test]
    fn non_kana_passthrough() {
        assert_eq!(kana_to_romaji("斉藤"), "斉藤");
        assert_eq!(kana_to_romaji("ABC"), "ABC");
        assert_eq!(kana_to_romaji("123"), "123");
    }

    #[test]
    fn mixed_kana_kanji() {
        assert_eq!(kana_to_romaji("斉藤ゆうき"), "斉藤yuki");
    }

    #[test]
    fn representative_names() {
        assert_eq!(kana_to_romaji("サトウケンジ"), "satokenji");
        assert_eq!(kana_to_romaji("トウキョウ"), "tokyo");
        assert_eq!(kana_to_romaji("オオサカ"), "osaka");
    }

    #[test]
    fn empty_string() {
        assert_eq!(kana_to_romaji(""), "");
    }

    #[test]
    fn foreign_sound_compounds() {
        assert_eq!(kana_to_romaji("ファ"), "fa");
        assert_eq!(kana_to_romaji("ティ"), "ti");
        assert_eq!(kana_to_romaji("ヴァ"), "va");
    }
}
