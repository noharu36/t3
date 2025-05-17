use bytes::BytesMut;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use t3::encode::encode_file;

fn bench_encode_data(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode");
    for n in [1, 10, 50, 100, 300, 512, 1024] {
        let data: Vec<u8> = vec![1u8; n * 1024 * 1024];
        group.bench_with_input(
            BenchmarkId::new("bench_encode_data", n),
            &data,
            |b, data| {
                b.iter(|| encode_file(BytesMut::from(&data[..])));
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_encode_data);
criterion_main!(benches);
