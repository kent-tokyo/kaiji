//! Check whether pairs of CJK names match under variant normalization.
//!
//! Usage:
//!   cargo run --example match_names
//!   cargo run --example match_names -- --width   # also normalize fullwidth/halfwidth

use std::env;

use kaiji::Normalizer;

fn main() {
    let args: Vec<String> = env::args().collect();
    let width = args.iter().any(|a| a == "--width" || a == "-w");

    let n = Normalizer::builder()
        .fold_variants(true)
        .strip_ivs(true)
        .width_normalization(width)
        .build();

    let pairs: &[(&str, &str, &str)] = &[
        // (label, input A, input B)
        ("斉 family", "斎藤一郎", "齋藤一郎"),
        ("斉 family", "斉藤一郎", "齊藤一郎"),
        ("辺 family", "渡辺花子", "渡邊花子"),
        ("辺 family", "渡辺花子", "渡邉花子"),
        ("吉 family", "𠮷野家", "吉野家"),
        ("広 family", "廣島太郎", "広島太郎"),
        ("関 family", "關西洋子", "関西洋子"),
        ("different", "斎藤", "佐藤"),
        ("different", "渡辺", "田中"),
    ];

    let col_w = 20usize;
    println!(
        "{:<col_w$}  {:<col_w$}  {:<col_w$}  Match?",
        "Input A", "Input B", "Normalized A"
    );
    println!("{}", "-".repeat(80));

    for (label, a, b) in pairs {
        let result = n.matches(a, b).unwrap_or(false);
        let norm_a = n.normalize(a).map(|s| s.into_owned()).unwrap_or_default();
        let mark = if result { "yes" } else { "no " };
        println!(
            "{:<col_w$}  {:<col_w$}  {:<col_w$}  {mark}   [{label}]",
            a, b, norm_a
        );
    }
}
