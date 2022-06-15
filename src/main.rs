use serde::{Deserialize, Serialize};
use serde_json;

use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

mod types;
// use types::{Album, Artist, Aspect, Song};

#[derive(Serialize, Deserialize, Debug)]
// struct Endsong {
//     entries: Vec<Entry>,
// }

// problem: null - https://stackoverflow.com/questions/44205435/how-to-deserialize-a-json-file-which-contains-null-values-using-serde
// Another way is to write our own deserialization routine for the field, which will accept null and turn it to something else of type String
struct Entry {
    ts: String,
    #[serde(skip_deserializing)]
    username: bool,
    #[serde(skip_deserializing)]
    platform: bool,
    ms_played: i32,
    #[serde(skip_deserializing)]
    ip_addr_decrypted: bool,
    #[serde(skip_deserializing)]
    user_agent_decrypted: bool,
    master_metadata_track_name: Option<String>,
    master_metadata_album_artist_name: Option<String>,
    master_metadata_album_album_name: Option<String>,
    spotify_track_uri: Option<String>,
    #[serde(skip_deserializing)]
    episode_name: bool,
    #[serde(skip_deserializing)]
    episode_show_name: bool,
    #[serde(skip_deserializing)]
    spotify_episode_uri: bool,
    #[serde(skip_deserializing)]
    reason_start: bool,
    #[serde(skip_deserializing)]
    reason_end: bool,
    #[serde(skip_deserializing)]
    shuffle: bool, // null, true, false
    #[serde(skip_deserializing)]
    skipped: bool, // null, true, false
    #[serde(skip_deserializing)]
    offline: bool, // null, true, false
    #[serde(skip_deserializing)]
    offline_timestamp: bool,
    #[serde(skip_deserializing)]
    incognito_mode: bool, // null, true, false
}

fn main() {
    // this is only temporary -> later on these files should be added by CLI args
    let paths: Vec<&str> = vec![
        "/home/filip/Other/SpotifyData/2021-07/endsong_0.json",
        "/home/filip/Other/SpotifyData/2021-07/single_entry.json",
    ];
    // let contents = fs::read_to_string(path[0]).expect("Something went wrong reading the file");

    // println!("{}", contents);

    let u = read_user_from_file(paths[0]).unwrap();
    println!("{:?}", u);

    // let song_test: Aspect::Song(String::from("Midnight Messiah"), Album(String::from("Bible of the Beast"), Artist(String::from("Powerwolf")), Artist(String::from("Powerwolf"))));
    types::run();
}

// https://docs.serde.rs/serde_json/fn.from_reader.html
fn read_user_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<Entry>, Box<dyn Error>> {
    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `User`.
    let u = serde_json::from_reader(reader)?;

    // Return the `User`.
    Ok(u)
}
