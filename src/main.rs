//! [![github]](https://github.com/Filip-Tomasko/rusty-endsong-parser/)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//!
//! CLI application with which you can analyze Spotify endsong.json files
#![deny(unsafe_code)]
// To require working docs
#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::invalid_codeblock_attributes,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::bare_urls
)]
#![warn(clippy::pedantic)]

mod display;
mod parse;
mod plot;
mod types;
mod ui;

use chrono::TimeZone;

use types::Aspect;
use types::AspectFull;
use types::SongEntries;

use parse::LOCATION_TZ;

/// Intializes the data,
/// tests some functions using [`test()`] and
/// starts the shell instance
fn main() {
    // different root path depending on my OS
    let root = match std::env::consts::OS {
        "windows" => "C:\\Temp\\Endsong\\",
        "macos" => "/Users/filip/Other/Endsong/",
        _ => "/mnt/c/temp/Endsong/",
    };
    let paths = [
        format!("{root}endsong_0.json"),
        // format!("{root}endsong_1.json"),
        // format!("{root}endsong_2.json"),
        // format!("{root}endsong_3.json"),
        // format!("{root}endsong_4.json"),
        // format!("{root}endsong_5.json"),
        // format!("{root}endsong_6.json"),
        // format!("{root}endsong_7.json"),
    ];

    let entries = SongEntries::new(&paths).unwrap();

    // test(&entries);
    // test_plot(&entries);

    ui::start(&entries);
}

/// tests various [`display`] functions
/// or its wrapper associated methods from [`SongEntries`]
#[allow(dead_code)]
fn test(entries: &SongEntries) {
    entries.print_top(&Aspect::default(), 10);
    entries.print_top(&Aspect::Albums, 10);
    entries.print_top(&Aspect::Artists, 10);

    let powerwolf = types::Artist::new(String::from("Powerwolf"));
    entries.print_top_from_artist(&Aspect::Songs, &powerwolf, 10);
    entries.print_top_from_artist(&Aspect::Albums, &powerwolf, 10);

    let coat = types::Album::from_str("Coat of Arms", "Sabaton");
    entries.print_top_from_album(&coat, 50);

    let final_solution = types::Song::from_str("The Final Solution", "Coat of Arms", "Sabaton");
    entries.print_aspect(&AspectFull::Artist(&types::Artist::from_str("Sabaton")));
    println!();
    entries.print_aspect(&AspectFull::Album(&coat));
    println!();
    entries.print_aspect(&AspectFull::Song(&final_solution));

    dbg!(entries.find().artist("Sabaton").unwrap());
    dbg!(entries.find().album("COAT OF ARMS", "sabaton").unwrap());
    dbg!(entries
        .find()
        .song_from_album("The FINAL SOLutiOn", "COAT OF ARMS", "sabaton",)
        .unwrap());
    match entries.find().artist("daduasdy712e qyw7") {
        Ok(art) => {
            dbg!(art);
        }
        Err(e) => {
            dbg!(e);
        }
    }
    // here to test whether it finds the multiple versions of this song (from many albums)
    // btw.. fuck Wizardthrone for releasing singles one after the other with each
    // containing all the songs that were in the previous one ffs
    dbg!(entries
        .find()
        .song("Frozen Winds Of Thyraxia", "Wizardthrone",)
        .unwrap());

    let start_date = LOCATION_TZ
        .datetime_from_str("2020-01-01T01:01:01Z", "%Y-%m-%dT%H:%M:%SZ")
        .unwrap();
    let end_date = LOCATION_TZ
        .datetime_from_str("2022-07-01T01:01:01Z", "%Y-%m-%dT%H:%M:%SZ")
        .unwrap();

    entries.print_aspect_date(&AspectFull::Artist(&powerwolf), &start_date, &end_date);
    entries.print_aspect_date(&AspectFull::Album(&coat), &start_date, &end_date);
    entries.print_aspect_date(&AspectFull::Song(&final_solution), &start_date, &end_date);

    assert_eq!(
        &entries.total_listening_time(),
        &entries.listening_time(&entries.first_date(), &entries.last_date())
    );

    let (time, start, end) = entries.max_listening_time(chrono::Duration::weeks(26 * 9));
    dbg!(time.num_minutes(), start.date_naive(), end.date_naive());

    dbg!(display::date::sum_plays(entries, &start, &end));
    display::date::print_time_played(entries, &start, &end);
    dbg!(entries.listening_time(&start, &end).num_minutes());
}

/// tests various [`plot`] functions
#[allow(dead_code)]
fn test_plot(entries: &SongEntries) {
    // plot::absolute::create(entries, &types::Artist::from_str("Sabaton"));

    let stand = types::Album::from_str("The Last Stand", "Sabaton");
    // plot::relative::to_all(entries, &coat);
    // plot::relative::to_artist(entries, &coat);

    // plot::single(plot::absolute::aspect(entries, &stand));
    plot::single(plot::absolute::aspect(entries, &stand));

    let eminem = types::Artist::from_str("Eminem");
    plot::compare(
        plot::relative::to_artist(entries, &stand),
        plot::relative::to_all(entries, &eminem),
    );
}
