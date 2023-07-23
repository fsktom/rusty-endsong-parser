use chrono::{DateTime, TimeZone};
use chrono_tz::Tz;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

// use endsong::plot;
#[allow(unused_imports)]
use endsong::types::{Album, Artist, Song, SongEntries};
use endsong::LOCATION_TZ;

/// # Arguments
/// * `usr_input` - in YYYY-MM-DD format or 'now' or 'start'
fn user_input_date_parser(usr_input: &str) -> Result<DateTime<Tz>, chrono::format::ParseError> {
    let date_str = match usr_input {
        "now" => {
            return Ok(LOCATION_TZ
                .timestamp_millis_opt(chrono::offset::Local::now().timestamp_millis())
                .unwrap())
        }
        // TODO! not hardcode this lol -> actual earlierst entry in endsong
        // -> problem with that: would have to pass &entries to this function
        // actually not big problem, I could even put LOCATION_TZ as a field of it
        // and not a constant :O
        "start" => String::from("1980-01-01T00:00:00Z"),
        // usr_input should be in YYYY-MM-DD format
        _ => format!("{usr_input}T00:00:00Z"),
    };

    // "%FT%TZ" is equivalent to "%Y-%m-%dT%H:%M:%SZ"
    // see <https://docs.rs/chrono/latest/chrono/format/strftime/index.html>
    LOCATION_TZ.datetime_from_str(&date_str, "%FT%TZ")
}

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
            black_box(entries.listening_time_date(&start, &end));
        })
    });
}

#[allow(dead_code)]
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

    let entries = black_box(SongEntries::new(&paths[7..=9]).unwrap());

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

fn parse(c: &mut Criterion) {
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

    c.bench_function("parse", |c| {
        c.iter(|| {
            black_box(endsong::types::SongEntries::new(&paths[..=1]).unwrap());
        })
    });
}

// criterion_group!(benches, lol);
// criterion_group!(benches, kekw);
criterion_group!(benches, parse);
criterion_main!(benches);
