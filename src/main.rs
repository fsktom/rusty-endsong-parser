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
    ];

    let entries = parse::parse(paths);

    println!(
        "{} - {} ({}) played on {} for {}ms || ID: {}",
        entries[2].artist,
        entries[2].track,
        entries[2].album,
        entries[2].timestamp,
        entries[2].ms_played,
        entries[2].id
    );

    // dbg!(entries.len());

    display::print_top(&entries, Aspect::Songs, 10000);
}
