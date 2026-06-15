//! Normalize CJK text line by line from stdin to stdout.
//!
//! Usage:
//!   echo "齋藤一郎" | cargo run --example normalize_cli
//!   cat names.csv | cargo run --example normalize_cli
//!   cargo run --example normalize_cli -- --width   # also convert fullwidth ASCII / halfwidth kana

use std::env;
use std::io::{self, BufRead, Write};

use kaiji::Normalizer;

fn main() {
    let args: Vec<String> = env::args().collect();
    let width = args.iter().any(|a| a == "--width" || a == "-w");

    let normalizer = Normalizer::builder()
        .fold_variants(true)
        .strip_ivs(true)
        .width_normalization(width)
        .build();

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("read error: {e}");
                std::process::exit(1);
            }
        };

        match normalizer.normalize(&line) {
            Ok(normalized) => {
                if let Err(e) = writeln!(out, "{normalized}") {
                    eprintln!("write error: {e}");
                    std::process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("normalize error: {e}");
                std::process::exit(1);
            }
        }
    }
}
