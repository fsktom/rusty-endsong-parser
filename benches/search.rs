use rusty_endsong_parser::{
    types::{Album, Artist, Song, SongEntries},
    ui::user_input_date_parser,
};

use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[allow(dead_code)]
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
        format!("{root}endsong_9.json"),
    ];

    let entries = black_box(SongEntries::new(&paths[..=2]).unwrap());

    c.bench_function("artists_vec", |c| {
        c.iter(|| {
            entries.artists();
        })
    });

    let powerwolf = black_box(Artist::new("Powerwolf"));
    c.bench_function("albums_vec", |c| {
        c.iter(|| {
            entries.albums(&powerwolf);
        })
    });

    c.bench_function("absolute", |c| {
        c.iter(|| {
            entries.traces().absolute(&powerwolf);
        })
    });
    c.bench_function("relative", |c| {
        c.iter(|| {
            entries.traces().relative(&powerwolf);
        })
    });

    let coat = black_box(Album::new("Coat of Arms", "Sabaton"));
    c.bench_function("relative_to_artist", |c| {
        c.iter(|| {
            entries.traces().relative_to_artist(&coat);
        })
    });

    let start = user_input_date_parser("2020-01-01").unwrap();
    let end = user_input_date_parser("2021-01-01").unwrap();
    c.bench_function("listening_time", |c| {
        c.iter(|| {
            entries.listening_time(&start, &end);
        })
    });
}

fn kekw(c: &mut Criterion) {
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
        format!("{root}endsong_9.json"),
    ];

    let entries = black_box(SongEntries::new(&paths[7..=7]).unwrap());

    let lth = Song::new(
        "Last Train Home",
        "Still Life (Talking)",
        "Pat Metheny Group",
    );
    c.bench_function("song_length", |c| {
        c.iter(|| {
            black_box(entries.song_length(&lth));
        })
    });

    c.bench_function("song_durations", |c| {
        c.iter(|| {
            black_box(entries.song_durations());
        })
    });
}

// criterion_group!(benches, lol);
criterion_group!(benches, kekw);
criterion_main!(benches);
