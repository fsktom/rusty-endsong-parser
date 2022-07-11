//! [![github]](https://github.com/Filip-Tomasko/rusty-endsong-parser/)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//!
//! CLI application with which you can analyze Spotify endsong.json files
#![forbid(unsafe_code)]
// To require working docs
#![forbid(missing_docs)]
#![forbid(clippy::missing_docs_in_private_items)]
#![forbid(rustdoc::broken_intra_doc_links)]
#![forbid(rustdoc::private_intra_doc_links)]
#![forbid(rustdoc::missing_crate_level_docs)]
#![forbid(rustdoc::invalid_codeblock_attributes)]
#![forbid(rustdoc::invalid_rust_codeblocks)]
#![forbid(rustdoc::bare_urls)]

mod display;
mod parse;
mod types;

use crate::types::Aspect;
use crate::types::AspectFull;
use crate::types::SongEntries;

/// Currently just tests various [crate::display] functions
/// after deserializing the endsong.json files using
/// [crate::parse::parse()] or its wrapper method
/// implemented in [SongEntries::new()]
fn main() {
    // this is only temporary -> later on these files should be added by CLI args
    // or both options!
    // benchmark! =>
    // 1) paths as CLI args
    // 2) paths as part of compiled source code!
    let root = "/home/filip/Other/SpotifyData/2022-06/";
    let paths: Vec<String> = vec![
        format!("{}endsong_0.json", root),
        // format!("{}endsong_1.json", root),
        // format!("{}endsong_2.json", root),
        // format!("{}endsong_3.json", root),
        // format!("{}endsong_4.json", root),
        // format!("{}endsong_5.json", root),
        // format!("{}endsong_6.json", root),
        // format!("{}endsong_7.json", root),
    ];

    let entries = SongEntries::new(paths);

    entries.print_top(Aspect::default(), 10);
    entries.print_top(Aspect::Albums, 10);
    entries.print_top(Aspect::Artists, 10);

    let powerwolf = types::Artist::new(String::from("Powerwolf"));
    entries.print_top_from_artist(Aspect::Songs, &powerwolf, 10);
    entries.print_top_from_artist(Aspect::Albums, &powerwolf, 10);

    let coat = types::Album::from_str("Coat of Arms", "Sabaton");
    entries.print_top_from_album(&coat, 50);

    let final_solution = types::Song::from_str("The Final Solution", "Coat of Arms", "Sabaton");
    entries.print_aspect(AspectFull::Artist(&types::Artist::from_str("Sabaton")));
    println!();
    entries.print_aspect(AspectFull::Album(&coat));
    println!();
    entries.print_aspect(AspectFull::Song(&final_solution));

    dbg!(display::find_artist(&entries, "Sabaton".to_string()));
    dbg!(display::find_album(
        &entries,
        "Coat of Arms".to_string(),
        "Sabaton".to_string()
    ));
    dbg!(display::find_song_from_album(
        &entries,
        "The FINAL SOLutiOn".to_string(),
        "COAT OF ARMS".to_string(),
        "sabaton".to_string(),
    ));
    dbg!(display::find_artist(
        &entries,
        "daduasdy712e qyw7".to_string()
    ));
    // here to test whether it finds the multiple versions of this song (from many albums)
    // btw.. fuck Wizardthrone for releasing singles one after the other with each
    // containing all the songs that were in the previous one ffs
    dbg!(display::find_song(
        &entries,
        "Frozen Winds Of Thyraxia".to_string(),
        "Wizardthrone".to_string(),
    ));

    // entries.print_aspect(AspectFull::Artist(&types::Artist::from_str("Wizardthrone")));
}
