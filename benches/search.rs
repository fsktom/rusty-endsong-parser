use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[allow(unused_imports)]
use endsong::prelude::*;
use itertools::Itertools;

fn paths(first: usize, last: usize) -> Vec<String> {
    let root = match std::env::consts::OS {
        "windows" => r"C:\Temp\Endsong\",
        "macos" => "/Users/filip/Other/Endsong/",
        _ => "/mnt/c/temp/Endsong/",
    };
    (first..=last)
        .map(|i| format!("{root}endsong_{i}.json"))
        .collect()
}

#[allow(dead_code)]
fn lol(c: &mut Criterion) {
    let entries = black_box(SongEntries::new(&paths(0, 2)).unwrap());

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

    let start = parse_date("2020-01-01").unwrap();
    let end = parse_date("2021-01-01").unwrap();
    c.bench_function("listening_time", |c| {
        c.iter(|| {
            black_box(gather::listening_time(entries.between(&start, &end)));
        })
    });
}

#[allow(dead_code)]
fn kekw(c: &mut Criterion) {
    let entries = black_box(SongEntries::new(&paths(7, 9)).unwrap());

    let lth = Song::new(
        "Last Train Home",
        "Still Life (Talking)",
        "Pat Metheny Group",
    );
    c.bench_function("song_length", |c| {
        c.iter(|| {
            black_box(entries.durations.get(&lth).unwrap());
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
    let paths = paths(0, 0);

    c.bench_function("parse", |c| {
        c.iter(|| {
            black_box(SongEntries::new(&paths).unwrap());
        })
    });

    c.bench_function("parse and filter", |c| {
        c.iter(|| {
            black_box(
                SongEntries::new(&paths)
                    .unwrap()
                    .filter(30, TimeDelta::seconds(10)),
            );
        })
    });

    // let a = SongEntries::new(&paths)
    //     .unwrap()
    //     .sum_different_capitalization()
    //     .filter(30, TimeDelta::seconds(10));
    // let b = SongEntries::new(&paths)
    //     .unwrap()
    //     .new_sum_different_capitalization()
    //     .filter(30, TimeDelta::seconds(10));
    // assert!(a.iter().eq(b.iter()));

    c.bench_function("parse, sum and filter", |c| {
        c.iter(|| {
            black_box(
                SongEntries::new(&paths)
                    .unwrap()
                    .sum_different_capitalization()
                    .filter(30, TimeDelta::seconds(10)),
            );
        })
    });

    // c.bench_function("new parse, sum and filter", |c| {
    //     c.iter(|| {
    //         black_box(
    //             SongEntries::new(&paths)
    //                 .unwrap()
    //                 .new_sum_different_capitalization()
    //                 .filter(30, TimeDelta::seconds(10)),
    //         );
    //     })
    // });
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
    let entries = black_box(SongEntries::new(&paths(0, 0)).unwrap());

    // c.bench_function("gather artists", |c| {
    //     c.iter(|| {
    //         black_box(gather::artists(&entries));
    //     })
    // });
    // c.bench_function("gather albums", |c| {
    //     c.iter(|| {
    //         black_box(gather::albums(&entries));
    //     })
    // });
    c.bench_function("gather songs summed across albums", |c| {
        c.iter(|| {
            black_box(gather::songs_summed_across_albums(&entries));
        })
    });
}

#[allow(dead_code)]
fn capitalization(c: &mut Criterion) {
    c.bench_function("parse and sum diff capitalization", |c| {
        c.iter(|| {
            black_box(
                SongEntries::new(&paths(0, 0))
                    .unwrap()
                    .sum_different_capitalization(),
            );
        })
    });
}

#[allow(dead_code)]
fn find(c: &mut Criterion) {
    let entries = black_box(SongEntries::new(&paths(0, 9)).unwrap());

    c.bench_function("find v1", |c| {
        c.iter(|| {
            black_box(entries.find().song_from_album(
                "MUKANJYO",
                "MUKANJYO",
                "Survive Said The Prophet",
            ));
        })
    });

    let art = Artist::new("survive said the prophet");
    c.bench_function("find artist v1", |c| {
        c.iter(|| {
            entries
                .iter()
                .find(|entry| art.is_entry_ignore_case(entry))
                .map(Artist::from)
        })
    });
    c.bench_function("find artist v2", |c| {
        c.iter(|| {
            entries
                .iter()
                .filter(|entry| art.is_entry_ignore_case(entry))
                .map(Artist::from)
                .unique()
                .collect_vec()
        })
    });
    c.bench_function("find song", |c| {
        c.iter(|| black_box(entries.find().song("mukanjyo", "survive said THE PROPHET")))
    });
}

// criterion_group!(benches, lol);
// criterion_group!(benches, kekw);
// criterion_group!(benches, parse);
// criterion_group!(benches, unique_sum);
criterion_group!(benches, gather);
// criterion_group!(benches, capitalization);
// criterion_group!(benches, find);
criterion_main!(benches);
