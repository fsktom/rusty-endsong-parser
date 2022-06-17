mod parse;
mod types;

fn main() {
    // this is only temporary -> later on these files should be added by CLI args
    let paths: Vec<String> = vec![
        "/home/filip/Other/SpotifyData/2021-07/endsong_0.json".to_string(),
        "/home/filip/Other/SpotifyData/2021-07/endsong_1.json".to_string(),
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

    println!("{}", entries.len());

    types::run();
}
