use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use kaiji::{KaijiIndex, NormalizerConfig};

fn make_corpus(n: usize) -> Vec<String> {
    // Mix of clean names and variant-form names
    let base = [
        "斎藤一郎", "渡辺花子", "佐藤次郎", "鈴木三郎", "田中四郎",
        "齋藤一郎", "渡邊花子", "廣島太郎", "關西次郎", "發展三郎",
    ];
    (0..n)
        .map(|i| format!("{}{}", base[i % base.len()], i))
        .collect()
}

fn bench_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("index_build");
    for size in [100usize, 1_000, 10_000] {
        let corpus = make_corpus(size);
        group.bench_with_input(BenchmarkId::from_parameter(size), &corpus, |b, corpus| {
            b.iter(|| {
                KaijiIndex::build(corpus.clone(), NormalizerConfig::default()).unwrap()
            });
        });
    }
    group.finish();
}

fn bench_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("index_search");
    for size in [100usize, 1_000, 10_000] {
        let corpus = make_corpus(size);
        let index = KaijiIndex::build(corpus, NormalizerConfig::default()).unwrap();
        group.bench_with_input(BenchmarkId::from_parameter(size), &index, |b, idx| {
            b.iter(|| idx.search("齋藤一郎", 0.7).unwrap());
        });
    }
    group.finish();
}

criterion_group!(benches, bench_build, bench_search);
criterion_main!(benches);
