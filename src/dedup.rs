use std::collections::HashMap;

use crate::config::NormalizerConfig;
use crate::normalize::normalize;

/// Group strings that normalize to the same canonical form.
///
/// Returns `(canonical_form, originals)` pairs in the order the canonical form
/// was first encountered. Within each group, originals preserve input order.
///
/// Use this to deduplicate records where the same entity appears under multiple
/// CJK variant spellings — e.g., 齋藤/斎藤/斉藤 all refer to the same person.
///
/// # Example
/// ```
/// use kaiji::{group_variants, NormalizerConfig};
///
/// let names: Vec<String> = ["齋藤一郎", "斎藤一郎", "斉藤一郎", "渡邊花子", "渡辺花子"]
///     .iter().map(|s| s.to_string()).collect();
/// let groups = group_variants(&names, &NormalizerConfig::default());
///
/// assert_eq!(groups[0].0, "斉藤一郎");
/// assert_eq!(groups[0].1.len(), 3);
/// assert_eq!(groups[1].0, "渡辺花子");
/// assert_eq!(groups[1].1.len(), 2);
/// ```
pub fn group_variants(strings: &[String], config: &NormalizerConfig) -> Vec<(String, Vec<String>)> {
    let mut order: Vec<String> = Vec::new();
    let mut groups: HashMap<String, Vec<String>> = HashMap::new();

    for s in strings {
        let canonical = normalize(s, config)
            .map(|c| c.into_owned())
            .unwrap_or_else(|_| s.clone());

        if !groups.contains_key(&canonical) {
            order.push(canonical.clone());
        }
        groups.entry(canonical).or_default().push(s.clone());
    }

    order
        .into_iter()
        .map(|canonical| {
            let members = groups.remove(&canonical).unwrap_or_default();
            (canonical, members)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> NormalizerConfig {
        NormalizerConfig::default()
    }

    fn strs(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn groups_saito_variants() {
        let input = strs(&["齋藤一郎", "斎藤一郎", "斉藤一郎"]);
        let groups = group_variants(&input, &cfg());
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].0, "斉藤一郎");
        assert_eq!(groups[0].1.len(), 3);
    }

    #[test]
    #[cfg(not(feature = "chinese"))]
    fn groups_watanabe_variants() {
        let input = strs(&["渡邊花子", "渡辺花子"]);
        let groups = group_variants(&input, &cfg());
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].0, "渡辺花子");
        assert_eq!(groups[0].1.len(), 2);
    }

    #[test]
    fn preserves_first_seen_order() {
        let input = strs(&["齋藤一郎", "渡邊花子", "斎藤一郎", "渡辺花子"]);
        let groups = group_variants(&input, &cfg());
        assert_eq!(groups.len(), 2);
        // First canonical seen is 斉藤一郎 (from 齋藤一郎)
        assert_eq!(groups[0].0, "斉藤一郎");
    }

    #[test]
    fn empty_input_returns_empty() {
        let groups = group_variants(&[], &cfg());
        assert!(groups.is_empty());
    }

    #[test]
    fn no_variants_each_in_own_group() {
        let input = strs(&["田中", "山田", "鈴木"]);
        let groups = group_variants(&input, &cfg());
        assert_eq!(groups.len(), 3);
    }

    #[test]
    fn preserves_originals_within_group() {
        let input = strs(&["齋藤一郎", "斎藤一郎"]);
        let groups = group_variants(&input, &cfg());
        assert_eq!(groups[0].1, vec!["齋藤一郎", "斎藤一郎"]);
    }
}
