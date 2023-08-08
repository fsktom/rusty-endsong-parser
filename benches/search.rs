use criterion::{black_box, criterion_group, criterion_main, Criterion};

// use endsong::plot;
#[allow(unused_imports)]
use endsong::prelude::*;

/// # Arguments
/// * `usr_input` - in YYYY-MM-DD format or 'now' or 'start'
fn user_input_date_parser(usr_input: &str) -> Result<DateTime<Local>, chrono::format::ParseError> {
    let date_str = match usr_input {
        "now" => return Ok(chrono::offset::Local::now()),
        "start" => String::from("1980-01-01T00:00:00Z"),
        // usr_input should be in YYYY-MM-DD format
        _ => format!("{usr_input}T00:00:00Z"),
    };

    // "%FT%TZ" is equivalent to "%Y-%m-%dT%H:%M:%SZ"
    // see <https://docs.rs/chrono/latest/chrono/format/strftime/index.html>
    Local.datetime_from_str(&date_str, "%FT%TZ")
}

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

#[allow(dead_code)]
fn lol(c: &mut Criterion) {
    let entries = black_box(SongEntries::new(&paths()[..=2]).unwrap());

    c.bench_function("artists_vec", |c| {
        c.iter(|| {
            black_box(entries.artists());
        })
    });

    let powerwolf = black_box(Artist::new("Powerwolf"));
    c.bench_function("albums_vec", |c| {
        c.iter(|| {
            black_box(entries.albums(&powerwolf));
        })
    });

    // c.bench_function("absolute", |c| {
    //     c.iter(|| {
    //         plot::absolute::aspect(&entries, &powerwolf);
    //     })
    // });
    // c.bench_function("relative", |c| {
    //     c.iter(|| {
    //         plot::relative::to_all(&entries, &powerwolf);
    //     })
    // });

    // let coat = black_box(Album::new("Coat of Arms", "Sabaton"));
    // c.bench_function("relative_to_artist", |c| {
    //     c.iter(|| {
    //         plot::relative::to_artist(&entries, &coat);
    //     })
    // });

    let start = user_input_date_parser("2020-01-01").unwrap();
    let end = user_input_date_parser("2021-01-01").unwrap();
    c.bench_function("listening_time", |c| {
        c.iter(|| {
            black_box(gather::listening_time(entries.between(&start, &end)));
        })
    });
}

#[allow(dead_code)]
fn kekw(c: &mut Criterion) {
    let entries = black_box(SongEntries::new(&paths()[7..=9]).unwrap());

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
    c.bench_function("find song", |c| {
        c.iter(|| {
            black_box(
                endsong::find::song_from_album(
                    &entries,
                    "Last Train Home",
                    "Still Life (Talking)",
                    "Pat Metheny Group",
                )
                .unwrap(),
            );
        })
    });
}

#[allow(dead_code)]
fn parse(c: &mut Criterion) {
    let paths = paths();

    c.bench_function("parse", |c| {
        c.iter(|| {
            black_box(SongEntries::new(&paths[..=1]).unwrap());
        })
    });

    let entries = black_box(SongEntries::new(&paths[..=1]).unwrap());

    c.bench_function("songs", |c| {
        c.iter(|| {
            black_box(gather::songs(&entries, true));
        })
    });
}

// not related to this at all but just curious xd
#[allow(dead_code)]
fn unique_sum(c: &mut Criterion) {
    use std::collections::HashSet;

    use itertools::Itertools;
    use rand::Rng;

    let nums = black_box(
        (0..100000)
            .map(|_| rand::thread_rng().gen_range(0..1000))
            .collect_vec(),
    );

    c.bench_function("naive sum", |c| {
        c.iter(|| {
            let mut sum = 0;
            let mut seen = Vec::<i32>::new();
            for n in nums.iter() {
                if !seen.contains(n) {
                    sum += n;
                    seen.push(*n);
                }
            }
        })
    });

    c.bench_function("ok sum", |c| {
        c.iter(|| {
            let mut sum = 0;
            let mut seen = HashSet::<i32>::new();
            for n in nums.iter() {
                if seen.contains(n) {
                    continue;
                }
                sum += n;
                seen.insert(*n);
            }
        })
    });

    c.bench_function("idk sum", |c| {
        c.iter(|| {
            let mut sum = 0;
            let mut seen = HashSet::<i32>::new();
            for n in nums.iter() {
                seen.insert(*n);
            }
            for n in seen.iter() {
                sum += n;
            }
        })
    });

    c.bench_function("probs best sum", |c| {
        c.iter(|| {
            let mut sum = 0;
            let mut seen = HashSet::<i32>::new();
            for n in nums.iter() {
                if seen.insert(*n) {
                    sum += n;
                }
            }
        })
    });

    c.bench_function("iter sum", |c| {
        c.iter(|| {
            nums.iter().unique().sum::<i32>();
        })
    });
}

#[allow(dead_code)]
fn gather(c: &mut Criterion) {
    let entries = black_box(SongEntries::new(&paths()[..=0]).unwrap());

    c.bench_function("gather artists", |c| {
        c.iter(|| {
            black_box(gather::artists(&entries));
        })
    });
    c.bench_function("gather albums", |c| {
        c.iter(|| {
            black_box(gather::albums(&entries));
        })
    });
    c.bench_function("gather songs", |c| {
        c.iter(|| {
            black_box(gather::songs(&entries, true));
        })
    });
}

// criterion_group!(benches, lol);
// criterion_group!(benches, kekw);
criterion_group!(benches, parse);
// criterion_group!(benches, unique_sum);
// criterion_group!(benches, gather);
criterion_main!(benches);
