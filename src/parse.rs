use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use chrono::DateTime;

use serde::{Deserialize, Serialize};
use serde_json;

use crate::types::SongEntry;

// https://stackoverflow.com/questions/44205435/how-to-deserialize-a-json-file-which-contains-null-values-using-serde
// null values are either skipped (defaulted to unit tuple or are an Option)
#[derive(Serialize, Deserialize, Debug)]
struct Entry {
    ts: String,
    #[serde(skip_deserializing)]
    username: (),
    #[serde(skip_deserializing)]
    platform: (),
    ms_played: i32,
    #[serde(skip_deserializing)]
    ip_addr_decrypted: (),
    #[serde(skip_deserializing)]
    user_agent_decrypted: (),
    master_metadata_track_name: Option<String>,
    master_metadata_album_artist_name: Option<String>,
    master_metadata_album_album_name: Option<String>,
    spotify_track_uri: Option<String>,
    #[serde(skip_deserializing)]
    episode_name: (),
    #[serde(skip_deserializing)]
    episode_show_name: (),
    #[serde(skip_deserializing)]
    spotify_episode_uri: (),
    #[serde(skip_deserializing)]
    reason_start: (),
    #[serde(skip_deserializing)]
    reason_end: (),
    #[serde(skip_deserializing)]
    shuffle: (),
    #[serde(skip_deserializing)]
    skipped: (),
    #[serde(skip_deserializing)]
    offline: (),
    #[serde(skip_deserializing)]
    offline_timestamp: (),
    #[serde(skip_deserializing)]
    incognito_mode: (),
}

fn parse_single(path: String) -> Vec<SongEntry> {
    let u = read_entries_from_file(path).unwrap();
    let mut songs: Vec<SongEntry> = Vec::new();
    let mut podcasts: Vec<Entry> = Vec::new();
    for entry in u {
        match entry_to_songentry(entry) {
            Ok(song) => songs.push(song),
            Err(pod) => podcasts.push(pod),
        }
    }

    println!("{:?}\nNum of song entries: {}", songs, songs.len());

    println!("Num of non-song? entries: {}", podcasts.len());

    songs
}

pub fn parse(paths: Vec<String>) -> Vec<SongEntry> {
    let mut song_entries: Vec<SongEntry> = Vec::new();
    for path in paths {
        let mut one = parse_single(path);
        song_entries.append(&mut one)
    }
    song_entries
}

// https://docs.serde.rs/serde_json/fn.from_reader.html
fn read_entries_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<Entry>, Box<dyn Error>> {
    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of tshe file as an instance of `User`.
    let full_entries = serde_json::from_reader(reader)?;

    // Return the `User`.
    Ok(full_entries)
}

fn entry_to_songentry(entry: Entry) -> Result<SongEntry, Entry> {
    // to remove podcast entries
    // if the track is null, so are album and artist
    if parse_option(entry.master_metadata_track_name.clone()) == "n/a" {
        return Err(entry);
    }
    Ok(SongEntry {
        // RFC 3339 is basically ISO 8601
        // and the timestamp in endsong.json is in
        // "2016-07-21T01:02:07Z" format
        timestamp: DateTime::parse_from_rfc3339(&entry.ts).unwrap(),
        ms_played: entry.ms_played as u32,
        track: parse_option(entry.master_metadata_track_name),
        album: parse_option(entry.master_metadata_album_album_name),
        artist: parse_option(entry.master_metadata_album_artist_name),
    })
}

fn parse_option(opt: Option<String>) -> String {
    match opt {
        Some(data) => data,
        None => String::from("n/a"),
    }
}
