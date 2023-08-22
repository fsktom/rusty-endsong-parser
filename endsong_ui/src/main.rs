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

/// Intializes the data,
/// tests some functions using [`test()`] and
/// starts the shell instance
fn main() {
    // different root path depending on my OS
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

    let entries = SongEntries::new(&paths[..=0])
        .unwrap()
        .sum_different_capitalization()
        .filter(30, Duration::seconds(10));

    // test(&entries);
    // test_two(&entries);
    // test_plot(&entries);

    ui::start(&entries);
}

/// tests various [`print`][crate::print] and [`endsong::gather`] functions
#[allow(dead_code)]
fn test(entries: &SongEntries) {
    print::top(entries, Aspect::default(), 10, false);
    print::top(entries, Aspect::Albums, 10, false);
    print::top(entries, Aspect::Artists, 10, false);

    let powerwolf = Artist::new("Powerwolf");
    print::top_from_artist(entries, Mode::Songs, &powerwolf, 10);
    print::top_from_artist(entries, Mode::Albums, &powerwolf, 10);

    let coat = Album::new("Coat of Arms", "Sabaton");
    print::top_from_album(entries, &coat, 50);

    let final_solution = Song::new("The Final Solution", "Coat of Arms", "Sabaton");
    print::aspect(entries, &AspectFull::Artist(&Artist::new("Sabaton")));
    println!();
    print::aspect(entries, &AspectFull::Album(&coat));
    println!();
    print::aspect(entries, &AspectFull::Song(&final_solution));

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

    print::aspect_date(
        entries,
        &AspectFull::Artist(&powerwolf),
        &start_date,
        &end_date,
    );
    print::aspect_date(entries, &AspectFull::Album(&coat), &start_date, &end_date);
    print::aspect_date(
        entries,
        &AspectFull::Song(&final_solution),
        &start_date,
        &end_date,
    );

    assert_eq!(
        gather::listening_time(entries),
        gather::listening_time(entries.between(&entries.first_date(), &entries.last_date()))
    );

    let (time, start, end) = entries.max_listening_time(chrono::Duration::weeks(26 * 9));
    dbg!(time.num_minutes(), start.date_naive(), end.date_naive());

    dbg!(gather::all_plays(entries.between(&start, &end)));
    print::time_played_date(entries, &start, &end);
    dbg!(gather::listening_time(entries.between(&start, &end)).num_minutes());

    print::aspect(
        entries,
        &AspectFull::Album(&Album::new("Built To Last", "HammerFall")),
    );
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
    let mut alb_dur = Duration::seconds(0);
    let ct_songs = entries.find().songs_from_album(&ct);
    for song in &ct_songs {
        println!(
            "{} - {}",
            song.name,
            entries.durations.get(song).unwrap().display()
        );
        alb_dur = alb_dur + *entries.durations.get(song).unwrap();
    }
    dbg!(alb_dur.display(), ct_songs.len());

    // dbg!(entries.len());
    // entries.filter(30, Duration::seconds(5));
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
    plot::single(trace::absolute(entries, &stand));

    let eminem = Artist::new("Eminem");
    plot::compare(
        trace::relative::to_artist(entries, &stand),
        trace::relative::to_all(entries, &eminem),
    );
}
