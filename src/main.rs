/// Module for printing parsed data to stdout
mod display;
mod parse;
mod types;

use crate::types::Aspect;

fn main() {
    // this is only temporary -> later on these files should be added by CLI args
    let root = "/home/filip/Other/SpotifyData/2021-07/";
    let paths: Vec<String> = vec![
        format!("{}endsong_0.json", root),
        format!("{}endsong_1.json", root),
        format!("{}endsong_2.json", root),
        format!("{}endsong_3.json", root),
        format!("{}endsong_4.json", root),
        format!("{}endsong_5.json", root),
        format!("{}endsong_6.json", root),
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
}
