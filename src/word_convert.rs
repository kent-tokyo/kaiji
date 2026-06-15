//! Stage 3: word-level Chinese Traditional↔Simplified conversion.
//!
//! Resolves disambiguation cases where one Simplified char maps to multiple
//! Traditional chars depending on word context — e.g. 发 → 髮 (hair) vs 發 (emit).
//! Uses a longest-match left-to-right scan over a built-in seed dictionary.

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use crate::config::ChineseConvertMode;

static SC_TO_TC: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
static TC_TO_SC: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
static SC_FIRST_CHARS: OnceLock<HashSet<char>> = OnceLock::new();
static TC_FIRST_CHARS: OnceLock<HashSet<char>> = OnceLock::new();

/// Maximum key length in chars. Keeps the inner probe loop bounded.
const MAX_WORD_LEN_CHARS: usize = 6;

fn build_sc_to_tc() -> HashMap<&'static str, &'static str> {
    // Disambiguation: SC words where a single simplified char maps to different
    // traditional chars depending on context.
    // Stage 2 does NOT fire on simplified input (its map is TC→SC direction),
    // so these keys reach Stage 3 unchanged.
    [
        // 发 (fā): 髮 (hair) vs 發 (send/emit/develop)
        // 皮肤/头皮 entries omitted — 发 not present
        ("头发",   "頭髮"),  // head hair
        ("理发",   "理髮"),  // haircut
        ("发型",   "髮型"),  // hairstyle
        ("发际",   "髮際"),  // hairline
        ("白发",   "白髮"),  // white hair
        ("出发",   "出發"),  // depart
        ("出发点", "出發點"), // starting point
        ("发展",   "發展"),  // develop / growth
        ("发现",   "發現"),  // discover
        ("发生",   "發生"),  // happen / occur
        ("发布",   "發布"),  // publish / release
        ("发射",   "發射"),  // launch / fire
        ("发言",   "發言"),  // speak / statement
        ("发表",   "發表"),  // publish / announce
        // 面 (miàn): 麵 (noodle/flour) vs 面 (face/aspect/surface)
        // 面孔/面前/面对 omitted — 面 as "face" stays unchanged
        ("面条",   "麵條"),  // noodles
        ("拉面",   "拉麵"),  // pulled noodles / ramen
        ("面粉",   "麵粉"),  // flour
        ("方便面", "方便麵"), // instant noodles
        ("挂面",   "掛麵"),  // dried noodles
        ("面包",   "麵包"),  // bread
        // 里 (lǐ): 裡 (inside) vs 里 (unit of distance ~500 m)
        // 公里/万里/一里 omitted — 里 as unit stays unchanged
        ("里面",   "裡面"),  // inside
        ("这里",   "這裡"),  // here
        ("那里",   "那裡"),  // there
        ("心里",   "心裡"),  // in one's heart
        ("哪里",   "哪裡"),  // where
        // 后 (hòu): 後 (after/behind) vs 后 (queen/empress)
        // 皇后/太后 omitted — 后 as "empress" stays unchanged
        ("以后",   "以後"),  // after / in the future
        ("然后",   "然後"),  // then / afterwards
        ("后来",   "後來"),  // later on
        ("之后",   "之後"),  // after that
        ("此后",   "此後"),  // hereafter
        ("今后",   "今後"),  // from now on
        ("前后",   "前後"),  // before and after
        ("后面",   "後面"),  // back side / behind
        ("后边",   "後邊"),  // rear
        // 干 (gān/gàn): 乾 (dry) vs 幹 (trunk/backbone/do) vs 干 (shield)
        // 若干/干涉/干扰 omitted — 干 in those senses stays unchanged
        ("干燥",   "乾燥"),  // dry (adjective)
        ("饼干",   "餅乾"),  // biscuit / cracker
        ("干净",   "乾淨"),  // clean
        ("晒干",   "曬乾"),  // sun-dry
        ("干部",   "幹部"),  // cadre / official
        ("骨干",   "骨幹"),  // backbone / mainstay
        ("树干",   "樹幹"),  // tree trunk
        // 系/係: 关系→關係 (system) vs 联系→聯繫 (contact)
        ("关系",   "關係"),  // relationship  (关→關, 系→係)
        ("联系",   "聯繫"),  // contact / link (联→聯, 系→繫)
        ("系统",   "系統"),  // system         (系 stays, 统→統)
        // 当 (dāng): mostly unambiguous but 当→當 is very common
        ("当时",   "當時"),  // at that time
        ("当然",   "當然"),  // of course
        ("当地",   "當地"),  // local
        ("适当",   "適當"),  // appropriate
        // 征 (zhēng): 征 (military expedition) vs 徵 (solicit/sign)
        // 征服/长征 omitted — 征 stays as 征 in TC
        ("征求",   "徵求"),  // solicit (opinions)
        ("象征",   "象徵"),  // symbol / symbolize
        ("特征",   "特徵"),  // characteristic / feature
        ("征兆",   "徵兆"),  // omen / sign
    ]
    .into_iter()
    .collect()
}

fn build_tc_to_sc() -> HashMap<&'static str, &'static str> {
    // Reverse direction: TC word forms for chars that Stage 2 may NOT have
    // converted at the character level (e.g. chars absent from Stage 2's map).
    // For most TC→SC cases Stage 2's character-level map is sufficient.
    [
        ("頭髮",   "头发"),
        ("理髮",   "理发"),
        ("髮型",   "发型"),
        ("白髮",   "白发"),
        ("出發",   "出发"),
        ("發展",   "发展"),
        ("發現",   "发现"),
        ("麵條",   "面条"),
        ("拉麵",   "拉面"),
        ("麵粉",   "面粉"),
        ("麵包",   "面包"),
        ("裡面",   "里面"),
        ("心裡",   "心里"),
        ("聯繫",   "联系"),
        ("骨幹",   "骨干"),
        ("幹部",   "干部"),
        ("徵求",   "征求"),
        ("象徵",   "象征"),
        ("特徵",   "特征"),
        ("徵兆",   "征兆"),
    ]
    .into_iter()
    .collect()
}

fn build_first_chars(map: &HashMap<&'static str, &'static str>) -> HashSet<char> {
    map.keys().filter_map(|k| k.chars().next()).collect()
}

fn sc_to_tc_map() -> &'static HashMap<&'static str, &'static str> {
    SC_TO_TC.get_or_init(build_sc_to_tc)
}

fn tc_to_sc_map() -> &'static HashMap<&'static str, &'static str> {
    TC_TO_SC.get_or_init(build_tc_to_sc)
}

fn sc_first_chars() -> &'static HashSet<char> {
    SC_FIRST_CHARS.get_or_init(|| build_first_chars(sc_to_tc_map()))
}

fn tc_first_chars() -> &'static HashSet<char> {
    TC_FIRST_CHARS.get_or_init(|| build_first_chars(tc_to_sc_map()))
}

/// Perform word-level Chinese Traditional↔Simplified conversion using a longest-match scan.
///
/// Returns [`Cow::Borrowed`] with zero allocation when:
/// - `mode` is [`ChineseConvertMode::Off`]
/// - no character in `input` begins a dictionary entry (first-char gate)
/// - characters begin entries but none actually match
///
/// `extra` supplies additional word mappings tried after the built-in dictionary.
/// Built-in entries take priority over `extra`.
pub fn convert_words<'a>(
    input: &'a str,
    mode: ChineseConvertMode,
    extra: Option<&HashMap<String, String>>,
) -> Cow<'a, str> {
    if mode == ChineseConvertMode::Off {
        return Cow::Borrowed(input);
    }

    let (map, gate): (&HashMap<&str, &str>, &HashSet<char>) = match mode {
        ChineseConvertMode::ToTraditional => (sc_to_tc_map(), sc_first_chars()),
        ChineseConvertMode::ToSimplified  => (tc_to_sc_map(), tc_first_chars()),
        ChineseConvertMode::Off           => unreachable!(),
    };

    // Per-call first-char set for the (typically small) extra dict.
    let extra_gate: HashSet<char> = extra
        .map(|e| e.keys().filter_map(|k| k.chars().next()).collect())
        .unwrap_or_default();

    let chars: Vec<(usize, char)> = input.char_indices().collect();
    let n = chars.len();
    let mut owned: Option<String> = None;
    let mut i = 0usize;

    while i < n {
        let (byte_pos, ch) = chars[i];

        // First-char gate: if ch cannot start any key, skip with no HashMap work.
        if !gate.contains(&ch) && !extra_gate.contains(&ch) {
            if let Some(ref mut buf) = owned {
                buf.push(ch);
            }
            i += 1;
            continue;
        }

        // Try match lengths from longest down to 1.
        let max_try = MAX_WORD_LEN_CHARS.min(n - i);
        let mut matched = false;

        for try_len in (1..=max_try).rev() {
            let end_byte = if i + try_len < n {
                chars[i + try_len].0
            } else {
                input.len()
            };
            let candidate = &input[byte_pos..end_byte];

            let replacement: Option<&str> = map
                .get(candidate)
                .copied()
                .or_else(|| extra.and_then(|e| e.get(candidate).map(String::as_str)));

            if let Some(repl) = replacement {
                if owned.is_none() {
                    owned = Some(input[..byte_pos].to_owned());
                }
                owned.as_mut().unwrap().push_str(repl);
                i += try_len;
                matched = true;
                break;
            }
        }

        if !matched {
            if let Some(ref mut buf) = owned {
                buf.push(ch);
            }
            i += 1;
        }
    }

    match owned {
        Some(s) => Cow::Owned(s),
        None    => Cow::Borrowed(input),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sc_to_tc(s: &str) -> Cow<'_, str> {
        convert_words(s, ChineseConvertMode::ToTraditional, None)
    }

    fn tc_to_sc(s: &str) -> Cow<'_, str> {
        convert_words(s, ChineseConvertMode::ToSimplified, None)
    }

    #[test]
    fn mode_off_returns_borrowed() {
        let input = "头发发展";
        let result = convert_words(input, ChineseConvertMode::Off, None);
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(result, input);
    }

    #[test]
    fn clean_ascii_returns_borrowed() {
        let result = sc_to_tc("hello world");
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn japanese_text_returns_borrowed() {
        let result = sc_to_tc("斎藤一郎");
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn no_match_returns_borrowed() {
        // SC text with no dict entries
        let result = sc_to_tc("皇后太后");
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn hair_vs_depart_disambiguation() {
        assert_eq!(sc_to_tc("头发"), "頭髮");
        assert_eq!(sc_to_tc("出发"), "出發");
    }

    #[test]
    fn noodle_vs_face_disambiguation() {
        assert_eq!(sc_to_tc("面条"), "麵條");
        // 面孔/面前 not in dict → stays as-is
        let result = sc_to_tc("面孔");
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn inside_vs_unit_disambiguation() {
        assert_eq!(sc_to_tc("里面"), "裡面");
        assert_eq!(sc_to_tc("这里"), "這裡");
        // 公里 not in dict → 里 stays as unit
        let result = sc_to_tc("公里");
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn after_vs_empress_disambiguation() {
        assert_eq!(sc_to_tc("以后"), "以後");
        assert_eq!(sc_to_tc("然后"), "然後");
        // 皇后 not in dict → 后 stays as empress
        let result = sc_to_tc("皇后");
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn dry_vs_cadre_disambiguation() {
        assert_eq!(sc_to_tc("干燥"), "乾燥");
        assert_eq!(sc_to_tc("干部"), "幹部");
    }

    #[test]
    fn longest_match_wins() {
        // "出发点" (3 chars) should match as a unit, not "出发" + "点"
        assert_eq!(sc_to_tc("出发点"), "出發點");
    }

    #[test]
    fn mixed_sentence() {
        // A sentence with multiple disambiguation cases
        assert_eq!(sc_to_tc("以后发展很好"), "以後發展很好");
    }

    #[test]
    fn tc_to_sc_basic() {
        assert_eq!(tc_to_sc("頭髮"), "头发");
        assert_eq!(tc_to_sc("骨幹"), "骨干");
    }

    #[test]
    fn custom_dict_extra() {
        let mut extra: HashMap<String, String> = HashMap::new();
        extra.insert("測試".to_owned(), "测试".to_owned());
        let result = convert_words("測試", ChineseConvertMode::ToSimplified, Some(&extra));
        assert_eq!(result, "测试");
    }

    #[test]
    fn custom_dict_does_not_override_builtin() {
        // Try to override a built-in entry with extra — built-in wins
        let mut extra: HashMap<String, String> = HashMap::new();
        extra.insert("头发".to_owned(), "WRONG".to_owned());
        let result = convert_words("头发", ChineseConvertMode::ToTraditional, Some(&extra));
        assert_eq!(result, "頭髮"); // built-in takes priority
    }
}
