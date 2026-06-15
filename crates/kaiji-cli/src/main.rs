use std::io::{self, BufRead, Write};

use clap::{Parser, Subcommand};
use kaiji::Normalizer;

#[derive(Parser)]
#[command(
    name = "kaiji",
    about = "CJK fuzzy match & normalization engine",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Normalize CJK text (reads lines from stdin, writes to stdout)
    Normalize {
        /// Convert fullwidth ASCII→halfwidth and halfwidth kana→fullwidth
        #[arg(short, long)]
        width: bool,

        /// Fold ASCII A–Z to a–z after normalization
        #[arg(long)]
        case_fold: bool,

        /// Apply Unicode NFKC normalization (requires --features nfkc at build time)
        #[arg(long)]
        nfkc: bool,

        /// Convert hiragana to katakana
        #[arg(long)]
        kana_to_katakana: bool,

        /// Convert katakana to hiragana (long vowel ー preserved)
        #[arg(long)]
        kana_to_hiragana: bool,

        /// Output format
        #[arg(long, value_name = "FORMAT", default_value = "text")]
        format: Format,
    },

    /// Check whether two strings match after CJK normalization
    Match {
        /// First string
        a: String,

        /// Second string
        b: String,

        /// Convert fullwidth/halfwidth before matching
        #[arg(short, long)]
        width: bool,
    },

    /// Compute Jaro-Winkler similarity score (0.0–1.0) between two strings
    Score {
        /// First string
        a: String,

        /// Second string
        b: String,

        /// Convert fullwidth/halfwidth before scoring
        #[arg(short, long)]
        width: bool,
    },
}

#[derive(Clone, clap::ValueEnum)]
enum Format {
    /// One normalized string per line (default)
    Text,
    /// Tab-separated: original\tnormalized
    Tsv,
    /// JSON array of {original, normalized} objects
    Json,
}

fn build_normalizer(width: bool, case_fold: bool, nfkc: bool, kana_to_katakana: bool, kana_to_hiragana: bool) -> Normalizer {
    let mut b = Normalizer::builder()
        .fold_variants(true)
        .strip_ivs(true)
        .width_normalization(width)
        .case_fold(case_fold)
        .kana_to_katakana(kana_to_katakana)
        .kana_to_hiragana(kana_to_hiragana);
    if nfkc {
        b = b.nfkc(true);
    }
    b.build()
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Normalize { width, case_fold, nfkc, kana_to_katakana, kana_to_hiragana, format } => {
            cmd_normalize(width, case_fold, nfkc, kana_to_katakana, kana_to_hiragana, format);
        }
        Command::Match { a, b, width } => {
            cmd_match(&a, &b, width);
        }
        Command::Score { a, b, width } => {
            cmd_score(&a, &b, width);
        }
    }
}

fn cmd_normalize(width: bool, case_fold: bool, nfkc: bool, kana_to_katakana: bool, kana_to_hiragana: bool, format: Format) {
    let normalizer = build_normalizer(width, case_fold, nfkc, kana_to_katakana, kana_to_hiragana);
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    let mut first_json = true;
    if matches!(format, Format::Json) {
        write!(out, "[").unwrap();
    }

    for line in stdin.lock().lines() {
        let original = line.unwrap_or_else(|e| {
            eprintln!("read error: {e}");
            std::process::exit(1);
        });

        let normalized = normalizer.normalize(&original).unwrap_or_else(|e| {
            eprintln!("normalize error: {e}");
            std::process::exit(1);
        });

        let result = match format {
            Format::Text => writeln!(out, "{normalized}"),
            Format::Tsv => writeln!(out, "{original}\t{normalized}"),
            Format::Json => {
                if !first_json {
                    writeln!(out, ",").ok();
                }
                first_json = false;
                let orig_escaped = original.replace('"', "\\\"");
                let norm_escaped = normalized.replace('"', "\\\"");
                write!(
                    out,
                    r#"{{"original":"{orig_escaped}","normalized":"{norm_escaped}"}}"#
                )
            }
        };

        if let Err(e) = result {
            eprintln!("write error: {e}");
            std::process::exit(1);
        }
    }

    if matches!(format, Format::Json) {
        writeln!(out, "\n]").unwrap();
    }
}

fn cmd_match(a: &str, b: &str, width: bool) {
    let n = build_normalizer(width, false, false, false, false);
    match n.matches(a, b) {
        Ok(true) => {
            println!("true");
            std::process::exit(0);
        }
        Ok(false) => {
            println!("false");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(2);
        }
    }
}

fn cmd_score(a: &str, b: &str, width: bool) {
    let n = build_normalizer(width, false, false, false, false);
    match n.similarity(a, b) {
        Ok(score) => println!("{score:.4}"),
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}
