//! [![github]](https://github.com/fsktom/rusty-endsong-parser/)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//!
//! CLI application with which you can analyze Spotify endsong.json files

// unsafe code is bad
#![deny(unsafe_code)]
// can be a pain, but it's worth it
// for stupid suggestions use #[allow(clippy::...)]
#![warn(clippy::pedantic)]
// because I want to be explicit when cloning is cheap
#![warn(clippy::clone_on_ref_ptr)]
// doc lints, checked when compiling/running clippy
#![warn(missing_docs, clippy::missing_docs_in_private_items)]
// other doc lints, only checked when building docs
// https://doc.rust-lang.org/rustdoc/lints.html
// other good ones are warn by default
#![warn(rustdoc::missing_crate_level_docs, rustdoc::unescaped_backticks)]

use endsong::prelude::*;
use endsong_ui::prelude::*;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

/// Intializes the data,
/// tests some functions using [`test()`] and
/// starts the shell instance
fn main() {
    let env = EnvFilter::builder()
        .with_default_directive(LevelFilter::ERROR.into())
        .from_env_lossy();
    tracing_subscriber::fmt().with_env_filter(env).init();

    // different root path depending on my OS
    let root = match std::env::consts::OS {
        "windows" => r"C:\Temp\Endsong\",
        "macos" => "/Users/filip/Other/Endsong/",
        _ => "/mnt/c/temp/Endsong/",
    };
    let last: u8 = 0;
    let paths: Vec<String> = (0..=last)
        .map(|i| format!("{root}endsong_{i}.json"))
        .collect();

    let entries = SongEntries::new(&paths)
        .unwrap_or_else(|e| panic!("{e}"))
        .sum_different_capitalization()
        .filter(30, TimeDelta::try_seconds(10).unwrap());

    // test(&entries);
    // test_two(&entries);
    // test_plot(&entries);

    ui::start(&entries);
}

/// tests various [`print`][crate::print] and [`endsong::gather`] functions
#[allow(dead_code)]
fn test(entries: &SongEntries) {
    print::top(entries, Aspect::Songs(false), 10);
    print::top(entries, Aspect::Albums, 10);
    print::top(entries, Aspect::Artists, 10);

    let powerwolf = Artist::new("Powerwolf");
    print::top_from_artist(entries, Mode::Songs, &powerwolf, 10);
    print::top_from_artist(entries, Mode::Albums, &powerwolf, 10);

    let coat = Album::new("Coat of Arms", "Sabaton");
    print::top_from_album(entries, &coat, 50);

    let final_solution = Song::new("The Final Solution", "Coat of Arms", "Sabaton");
    print::aspect(entries, &Artist::new("Sabaton"));
    println!();
    print::aspect(entries, &coat);
    println!();
    print::aspect(entries, &final_solution);

    dbg!(entries.find().artist("Sabaton").unwrap());
    dbg!(entries.find().album("COAT OF ARMS", "sabaton").unwrap());
    dbg!(entries
        .find()
        .song_from_album("The FINAL SOLutiOn", "COAT OF ARMS", "sabaton",)
        .unwrap());
    match entries.find().artist("daduasdy712e qyw7") {
        Some(art) => {
            dbg!(art);
        }
        None => {
            dbg!("nope");
        }
    }
    // here to test whether it finds the multiple versions of this song (from many albums)
    // btw.. fuck Wizardthrone for releasing singles one after the other with each
    // containing all the songs that were in the previous one ffs
    dbg!(entries
        .find()
        .song("Frozen Winds Of Thyraxia", "Wizardthrone",)
        .unwrap());

    let start_date = parse_date("2020-01-01").unwrap();
    let end_date = parse_date("2022-07-01").unwrap();

    print::aspect_date(entries, &powerwolf, &start_date, &end_date);
    print::aspect_date(entries, &coat, &start_date, &end_date);
    print::aspect_date(entries, &final_solution, &start_date, &end_date);

    assert_eq!(
        gather::total_listening_time(entries),
        gather::total_listening_time(entries.between(&entries.first_date(), &entries.last_date()))
    );

    let (time, start, end) = entries.max_listening_time(TimeDelta::try_weeks(26 * 9).unwrap());
    dbg!(time.num_minutes(), start.date_naive(), end.date_naive());

    dbg!(gather::all_plays(entries.between(&start, &end)));
    print::time_played_date(entries, &start, &end);
    dbg!(gather::total_listening_time(entries.between(&start, &end)).num_minutes());

    print::aspect(entries, &Album::new("Built To Last", "HammerFall"));
}

/// another test function
#[allow(dead_code)]
fn test_two(entries: &SongEntries) {
    let s = Song::new("STYX HELIX", "eYe's", "MYTH & ROID");
    assert!(entries
        .find()
        .song_from_album("STYX HELIX", "eYe's", "MYTH & ROID")
        .is_some());
    let a = entries.durations.get(&s).unwrap();
    dbg!(a.num_minutes(), a.num_seconds() - a.num_minutes() * 60);
    dbg!(a.display());

    let ct = Album::new("Waking The Fallen", "Avenged Sevenfold");
    let mut alb_dur = TimeDelta::try_seconds(0).unwrap();
    let ct_songs = get_sorted_list(gather::songs_from(entries, &ct));
    for song in &ct_songs {
        println!(
            "{} - {}",
            song.name,
            entries.durations.get(song).unwrap().display()
        );
        alb_dur += *entries.durations.get(song).unwrap();
    }
    dbg!(alb_dur.display(), ct_songs.len());

    // dbg!(entries.len());
    // entries.filter(30, TimeDelta::try_seconds(5).unwrap());
    // dbg!(entries.len());
}

/// tests various [`plot`] functions
#[allow(dead_code)]
fn test_plot(entries: &SongEntries) {
    // plot::absolute::create(entries, &types::Artist::from_str("Sabaton"));

    let stand = Album::new("The Last Stand", "Sabaton");
    // plot::relative::to_all(entries, &coat);
    // plot::relative::to_artist(entries, &coat);

    // plot::single(plot::absolute::aspect(entries, &stand));
    plot::single((trace::absolute(entries, &stand), String::from("test")));

    let eminem = Artist::new("Eminem");
    plot::compare(
        (
            trace::relative::to_artist(entries, &stand),
            String::from("test"),
        ),
        (
            trace::relative::to_all(entries, &eminem),
            String::from("test"),
        ),
    );
}
