use bytes::BytesMut;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use t3::encode::encode_file;
use tokio::runtime::Runtime;

fn bench_encode_data(c: &mut Criterion) {
    let data: &[u8] = &[1u8; 1024 * 1024 * 1024];
    c.bench_with_input(
        BenchmarkId::new("encode_small_data", "x"),
        &data,
        |b, &data| {
            b.to_async(Runtime::new().unwrap())
                .iter(|| encode_file(BytesMut::from(data)));
        },
    );
}

criterion_group!(benches, bench_encode_data);
criterion_main!(benches);
