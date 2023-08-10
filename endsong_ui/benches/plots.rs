use criterion::{black_box, criterion_group, criterion_main, Criterion};
use endsong::prelude::*;

use endsong_ui::trace;

fn paths() -> [String; 10] {
    let root = match std::env::consts::OS {
        "windows" => r"C:\\Temp\\Endsong\\",
        "macos" => "/Users/filip/Other/Endsong/",
        _ => "/mnt/c/temp/Endsong/",
    };
    [
        format!("{root}endsong_0.json"),
        format!("{root}endsong_1.json"),
        format!("{root}endsong_2.json"),
        format!("{root}endsong_3.json"),
        format!("{root}endsong_4.json"),
        format!("{root}endsong_5.json"),
        format!("{root}endsong_6.json"),
        format!("{root}endsong_7.json"),
        format!("{root}endsong_8.json"),
        format!("{root}endsong_9.json"),
    ]
}

fn traces(c: &mut Criterion) {
    let entries = black_box(SongEntries::new(&paths()[..=2]).unwrap());

    let powerwolf = black_box(Artist::new("Powerwolf"));
    let coat = black_box(Album::new("Carolus Rex", "Sabaton"));
    let spart = black_box(Song::new("Sparta", "The Last Stand", "Sabaton"));

    c.bench_function("absolute", |c| {
        c.iter(|| {
            trace::absolute(&entries, &powerwolf);
        })
    });
    c.bench_function("relative", |c| {
        c.iter(|| {
            trace::relative::to_all(&entries, &powerwolf);
        })
    });

    c.bench_function("relative_to_artist", |c| {
        c.iter(|| {
            trace::relative::to_artist(&entries, &coat);
        })
    });

    c.bench_function("relative_to_album", |c| {
        c.iter(|| {
            trace::relative::to_album(&entries, &spart);
        })
    });
}

criterion_group!(benches, traces);
criterion_main!(benches);
