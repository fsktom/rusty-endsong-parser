//! [![github]](https://github.com/Filip-Tomasko/rusty-endsong-parser/)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//!
//! CLI application with which you can analyze Spotify endsong.json files
#![deny(unsafe_code)]
// To require working docs
#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]
#![deny(rustdoc::missing_crate_level_docs)]
#![deny(rustdoc::invalid_codeblock_attributes)]
#![deny(rustdoc::invalid_rust_codeblocks)]
#![deny(rustdoc::bare_urls)]
#![deny(clippy::pedantic)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]

mod display;
mod parse;
mod types;
mod ui;

use chrono::TimeZone;
use types::Aspect;
use types::AspectFull;
use types::SongEntries;

use crate::parse::LOCATION_TZ;

/// Intializes the data,
/// tests some functions using [test()] and
/// starts the shell instance
fn main() {
    // this is only temporary -> later on these files should be added by CLI args
    // or both options!
    // benchmark! =>
    // 1) paths as CLI args
    // 2) paths as part of compiled source code!
    // let root = "/home/filip/Other/SpotifyData/2022-06/";
    let root = "/Users/filip/Other/Endsong/";
    let paths: Vec<String> = vec![
        format!("{root}endsong_0.json"),
        // format!("{}endsong_1.json", root),
        // format!("{}endsong_2.json", root),
        // format!("{}endsong_3.json", root),
        // format!("{}endsong_4.json", root),
        // format!("{}endsong_5.json", root),
        // format!("{}endsong_6.json", root),
        // format!("{}endsong_7.json", root),
    ];

    let entries = SongEntries::new(paths);

    test(&entries);

    ui::start(&entries);
}

/// tests various [`crate::display`] functions
/// or its wrapper associated methods from [`SongEntries`]
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
        .song(
            "Frozen Winds Of Thyraxia".to_string(),
            "Wizardthrone".to_string(),
        )
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
}
