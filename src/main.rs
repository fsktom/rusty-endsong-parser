mod parse;
mod types;

fn main() {
    // this is only temporary -> later on these files should be added by CLI args
    let paths: Vec<&str> = vec![
        "/home/filip/Other/SpotifyData/2021-07/endsong_0.json",
        "/home/filip/Other/SpotifyData/2021-07/single_entry.json",
    ];

    let entries = parse::parse(paths);

    println!(
        "{} - {} ({}) played on {} for {}ms",
        entries[2].artist,
        entries[2].track,
        entries[2].album,
        entries[2].timestamp,
        entries[2].ms_played
    );

    types::run();
}
