use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn lol(c: &mut Criterion) {
    let b: [u8; 3] = black_box([3, 4, 49]);

    fn add(b: [u8; 3]) -> u8 {
        let mut total = 0;
        for e in b {
            total += e
        }
        total
    }

    c.bench_function("what", |c| c.iter(|| add(b)));
}

criterion_group!(benches, lol);
criterion_main!(benches);
