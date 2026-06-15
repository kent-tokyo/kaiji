use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use kaiji::{Normalizer, NormalizerConfig, normalize};

// Representative inputs: clean (no substitution) and dirty (all variants)
const CLEAN_SHORT: &str = "斉藤一郎";
const DIRTY_SHORT: &str = "齋藤一郎"; // 齋 → 斉

const CLEAN_MEDIUM: &str = "田中太郎渡辺花子斉藤健山田美咲中村誠鈴木洋子加藤幸子伊藤明小林純";
const DIRTY_MEDIUM: &str = "田中太郎渡邊花子齋藤健山田美咲中村誠鈴木洋子加藤幸子伊藤明小林純";

fn make_long(template: &str, repeat: usize) -> String {
    template.repeat(repeat)
}

fn bench_normalize(c: &mut Criterion) {
    let cfg = NormalizerConfig::default();

    let mut group = c.benchmark_group("normalize/single_char");
    for (label, input) in [("clean", "斉"), ("dirty", "齋")] {
        group.bench_with_input(BenchmarkId::new(label, input), input, |b, s| {
            b.iter(|| normalize(std::hint::black_box(s), &cfg).unwrap());
        });
    }
    group.finish();

    let mut group = c.benchmark_group("normalize/short_string");
    for (label, input) in [("clean", CLEAN_SHORT), ("dirty", DIRTY_SHORT)] {
        group.bench_with_input(BenchmarkId::new(label, input), input, |b, s| {
            b.iter(|| normalize(std::hint::black_box(s), &cfg).unwrap());
        });
    }
    group.finish();

    let mut group = c.benchmark_group("normalize/medium_string");
    for (label, input) in [("clean", CLEAN_MEDIUM), ("dirty", DIRTY_MEDIUM)] {
        group.bench_with_input(BenchmarkId::new(label, label), input, |b, s| {
            b.iter(|| normalize(std::hint::black_box(s), &cfg).unwrap());
        });
    }
    group.finish();

    let long_clean = make_long(CLEAN_MEDIUM, 200); // ~5 000 chars
    let long_dirty = make_long(DIRTY_MEDIUM, 200);

    let mut group = c.benchmark_group("normalize/long_string_5k_chars");
    for (label, input) in [("clean", &long_clean), ("dirty", &long_dirty)] {
        group.bench_with_input(BenchmarkId::new(label, label), input.as_str(), |b, s| {
            b.iter(|| normalize(std::hint::black_box(s), &cfg).unwrap());
        });
    }
    group.finish();
}

fn bench_width(c: &mut Criterion) {
    // Stage 1 width normalization benchmarks using the Normalizer builder API.
    let n = Normalizer::builder()
        .width_normalization(true)
        .fold_variants(false)
        .strip_ivs(false)
        .build();

    let n_full = Normalizer::builder()
        .width_normalization(true)
        .fold_variants(true)
        .strip_ivs(true)
        .build();

    let mut group = c.benchmark_group("width/stage1");

    // Fullwidth ASCII — every character must be converted
    let fullwidth_ascii = "ＡＢＣＤＥＦＧＨＩＪＫＬＭＮＯＰＱＲＳＴＵＶＷＸＹＺａｂｃ";
    group.bench_with_input(
        BenchmarkId::new("fullwidth_ascii", "26+3 chars"),
        fullwidth_ascii,
        |b, s| {
            b.iter(|| n.normalize(std::hint::black_box(s)).unwrap());
        },
    );

    // Halfwidth katakana with dakuten — look-ahead composition per pair
    let halfwidth_kana_dakuten = "ｶﾞｷﾞｸﾞｹﾞｺﾞｻﾞｼﾞｽﾞｾﾞｿﾞ";
    group.bench_with_input(
        BenchmarkId::new("halfwidth_kana_dakuten", "10 pairs"),
        halfwidth_kana_dakuten,
        |b, s| {
            b.iter(|| n.normalize(std::hint::black_box(s)).unwrap());
        },
    );

    // Zero-copy fast path — pure ASCII, no conversion needed → returns Borrowed
    let plain_ascii = "Hello, world! This is a plain ASCII string with no CJK chars.";
    group.bench_with_input(
        BenchmarkId::new("plain_ascii_zero_copy", plain_ascii.len()),
        plain_ascii,
        |b, s| {
            b.iter(|| n.normalize(std::hint::black_box(s)).unwrap());
        },
    );

    // Full pipeline: width + variant fold (Stage 1 + Stage 2)
    let mixed = "ＡＢＣ齋藤一郎ｶﾞｷﾞ";
    group.bench_with_input(
        BenchmarkId::new("full_pipeline_mixed", mixed),
        mixed,
        |b, s| {
            b.iter(|| n_full.normalize(std::hint::black_box(s)).unwrap());
        },
    );

    group.finish();
}

criterion_group!(benches, bench_normalize, bench_width);
criterion_main!(benches);
