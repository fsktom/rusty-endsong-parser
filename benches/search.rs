use rusty_endsong_parser::types::{Artist, SongEntries};

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn lol(c: &mut Criterion) {
    let root = match std::env::consts::OS {
        "windows" => r"C:\\Temp\\Endsong\\",
        "macos" => "/Users/filip/Other/Endsong/",
        _ => "/mnt/c/temp/Endsong/",
    };
    let paths = [
        format!("{root}endsong_0.json"),
        format!("{root}endsong_1.json"),
        format!("{root}endsong_2.json"),
        format!("{root}endsong_3.json"),
        format!("{root}endsong_4.json"),
        format!("{root}endsong_5.json"),
        format!("{root}endsong_6.json"),
        format!("{root}endsong_7.json"),
        format!("{root}endsong_8.json"),
    ];

    let entries = black_box(SongEntries::new(&paths[..=2]).unwrap());

    c.bench_function("absolute", |c| {
        c.iter(|| {
            entries.traces().absolute(&Artist::new("Powerwolf"));
        })
    });

    c.bench_function("relative", |c| {
        c.iter(|| {
            entries.traces().relative(&Artist::new("Powerwolf"));
        })
    });
}

criterion_group!(benches, lol);
criterion_main!(benches);
