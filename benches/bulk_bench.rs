use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use kaiji::{NormalizerConfig, normalize};

// Simulate a CSV column of names (姓名) with mixed variant / canonical chars.
fn make_name_rows(count: usize) -> Vec<String> {
    let templates = [
        "齋藤一郎",
        "渡邊花子",
        "斎藤健",
        "廣島太郎",
        "關西洋子",
        "𠮷野家彦",
        "田中誠",
        "鈴木純子",
        "伊藤明",
        "山田美咲",
    ];
    (0..count)
        .map(|i| templates[i % templates.len()].to_owned())
        .collect()
}

fn bench_bulk_normalize(c: &mut Criterion) {
    let cfg = NormalizerConfig::default();

    for &n in &[1_000usize, 100_000, 1_000_000] {
        let rows = make_name_rows(n);
        let total_bytes: u64 = rows.iter().map(|s| s.len() as u64).sum();

        let mut group = c.benchmark_group("bulk_normalize");
        group.throughput(Throughput::Bytes(total_bytes));
        group.sample_size(10);

        group.bench_with_input(BenchmarkId::new("rows", n), &rows, |b, rows| {
            b.iter(|| {
                let mut count = 0usize;
                for row in rows {
                    let _ = normalize(std::hint::black_box(row.as_str()), &cfg).unwrap();
                    count += 1;
                }
                count
            });
        });
        group.finish();
    }
}

criterion_group!(benches, bench_bulk_normalize);
criterion_main!(benches);
