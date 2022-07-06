//! [![github]](https://github.com/Filip-Tomasko/rusty-endsong-parser/)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//!
//! Module for printing parsed data to stdout
mod display;
mod parse;
mod types;

use crate::types::Aspect;
use crate::types::AspectFull;

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

    let entries = parse::parse(paths);

    display::print_top(&entries, Aspect::default(), 10);
    display::print_top(&entries, Aspect::Albums, 10);
    display::print_top(&entries, Aspect::Artists, 10);

    let powerwolf = types::Artist {
        name: String::from("Powerwolf"),
    };
    display::print_top_from_artist(&entries, Aspect::Songs, &powerwolf, 10);
    display::print_top_from_artist(&entries, Aspect::Albums, &powerwolf, 10);

    let coat = types::Album {
        name: String::from("Coat of Arms"),
        artist: types::Artist {
            name: "Sabaton".to_string(),
        },
    };
    display::print_top_from_album(&entries, Aspect::Songs, &coat, 50);

    let final_solution = types::Song {
        name: String::from("The Final Solution"),
        album: coat.clone(),
    };
    display::print_aspect(
        &entries,
        AspectFull::Artist(&types::Artist {
            name: "Sabaton".to_string(),
        }),
    );
    println!();
    display::print_aspect(&entries, AspectFull::Album(&coat));
    println!();
    display::print_aspect(&entries, AspectFull::Song(&final_solution));
}
