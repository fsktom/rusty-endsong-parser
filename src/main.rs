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
/// [crate::parse::parse()]
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

    display::print_top(&entries, Aspect::default(), 10);
    display::print_top(&entries, Aspect::Albums, 10);
    display::print_top(&entries, Aspect::Artists, 10);

    let powerwolf = types::Artist::new(String::from("Powerwolf"));
    display::print_top_from_artist(&entries, Aspect::Songs, &powerwolf, 10);
    display::print_top_from_artist(&entries, Aspect::Albums, &powerwolf, 10);

    let coat = types::Album::from_str("Coat of Arms", "Sabaton");
    display::print_top_from_album(&entries, &coat, 50);

    let final_solution = types::Song::from_str("The Final Solution", "Coat of Arms", "Sabaton");
    display::print_aspect(
        &entries,
        AspectFull::Artist(&types::Artist::from_str("Sabaton")),
    );
    println!();
    display::print_aspect(&entries, AspectFull::Album(&coat));
    println!();
    display::print_aspect(&entries, AspectFull::Song(&final_solution));
}
