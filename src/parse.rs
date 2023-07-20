//! Module responsible for deserializing the endsong.json files
//! into usable Rust data types
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use chrono::{DateTime, TimeZone};

use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

use crate::types::{PodEntry, SongEntry};

/// responsible for time zone handling
///
/// see issue #4 <https://github.com/Filip-Tomasko/rusty-endsong-parser/issues/4>
///
/// used for parsing the timestamp in [`parse`][self]
pub const LOCATION_TZ: chrono_tz::Tz = chrono_tz::Europe::Berlin;

// https://stackoverflow.com/questions/44205435/how-to-deserialize-a-json-file-which-contains-null-values-using-serde
// null values are either skipped (defaulted to unit tuple or are an Option)
/// General/raw struct for a single entry in endsong.json
/// (which are an array of such structs)
///
/// Raw because it's directly the deserialization from endsong.json
///
/// These are later "converted" to
/// [`crate::types::SongEntry`] if they represent a song or to
/// [`crate::types::PodcastEntry`] if they represent a podcast (TBD)
#[derive(Serialize, Deserialize, Debug)]
pub struct Entry {
    /// timestamp in `"YYY-MM-DD 13:30:30"` format
    ts: String,
    /// Skipped
    #[serde(skip_deserializing)]
    username: (),
    /// Skipped for now: maybe use it for sth
    #[serde(skip_deserializing)]
    platform: String,
    /// Miliseconds the song has been played for
    ms_played: i64,
    /// Skipped
    #[serde(skip_deserializing)]
    conn_country: (),
    /// Skipped
    #[serde(skip_deserializing)]
    ip_addr_decrypted: (),
    /// Skipped
    #[serde(skip_deserializing)]
    user_agent_decrypted: (),
    /// Name of the song
    ///
    /// Option because the field will be empty if it's a podcast
    master_metadata_track_name: Option<String>,
    /// Name of the artist
    ///
    /// Option because the field will be empty if it's a podcast
    master_metadata_album_artist_name: Option<String>,
    /// Name of the album
    ///
    /// Option because the field will be empty if it's a podcast
    master_metadata_album_album_name: Option<String>,
    /// Spotify URI (ID)
    spotify_track_uri: Option<String>,
    /// TBD: Podcast stuff
    #[serde(skip_deserializing)]
    episode_name: (),
    /// TBD: Podcast stuff
    #[serde(skip_deserializing)]
    /// TBD: Podcast stuff
    episode_show_name: (),
    #[serde(skip_deserializing)]
    /// TBD: Podcast stuff
    spotify_episode_uri: (),
    /// Skipped for now: maybe use it for sth
    #[serde(skip_deserializing)]
    reason_start: String,
    /// Skipped for now: maybe use it for sth
    #[serde(skip_deserializing)]
    reason_end: String,
    /// Skipped for now: maybe use it for sth
    #[serde(skip_deserializing)]
    shuffle: bool,
    /// Skipped for now: maybe use it for sth
    #[serde(skip_deserializing)]
    skipped: Option<bool>,
    /// Skipped
    #[serde(skip_deserializing)]
    offline: (),
    /// Skipped
    #[serde(skip_deserializing)]
    offline_timestamp: (),
    /// Skipped
    #[serde(skip_deserializing)]
    incognito_mode: (),
}

/// Parses a single `endsong.json` file into a usable format
fn parse_single<P: AsRef<Path>>(path: P) -> Result<Vec<SongEntry>, Box<dyn Error>> {
    let u = read_entries_from_file(path)?;
    // at least for me: about 15.8k-15.95k entries per file
    // to prevent reallocations?
    let mut songs: Vec<SongEntry> = Vec::with_capacity(16_000);
    let mut podcasts: Vec<PodEntry> = Vec::with_capacity(1_000);
    for entry in u {
        match entry_to_songentry(entry) {
            Ok(song) => songs.push(song),
            Err(pod) => podcasts.push(pod),
        }
    }

    Ok(songs)
}

/// Main parsing function that parses many `endsong.json` files
pub fn parse<P: AsRef<Path>>(paths: &[P]) -> Result<Vec<SongEntry>, Box<dyn Error>> {
    // at least for me: about 15.8k-15.95k entries per file
    // to prevent reallocations?
    let mut song_entries: Vec<SongEntry> = Vec::with_capacity(16_000 * paths.len());
    for path in paths {
        let mut one = parse_single(path)?;
        song_entries.append(&mut one);
    }

    song_entries.sort_unstable_by(|a, b| a.timestamp.cmp(&b.timestamp));

    Ok(song_entries)
}

// https://docs.serde.rs/serde_json/fn.from_reader.html
/// Responsible for parsing the json into a vector of the general [Entry]
fn read_entries_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<Entry>, Box<dyn Error>> {
    // https://github.com/serde-rs/json/issues/160#issuecomment-253446892
    let mut file_contents = String::new();
    File::open(path)?.read_to_string(&mut file_contents)?;
    let full_entries: Vec<Entry> = serde_json::from_str(&file_contents)?;

    // Return entries
    Ok(full_entries)
}

/// Converts the genral [Entry] to a more specific [`SongEntry`]
fn entry_to_songentry(entry: Entry) -> Result<SongEntry, PodEntry> {
    // to remove podcast entries
    // if the track is null, so are album and artist
    if parse_option(entry.master_metadata_track_name.clone()) == "n/a" {
        // TODO! properly... not just a placeholder
        // bc clippy::pednatic complained about returning
        // big Result<SongEntry, Entry> with Entry being biiig
        let pod = PodEntry {
            id: "666".to_string(),
        };
        return Err(pod);
    }
    Ok(SongEntry {
        timestamp: parse_date(&entry.ts),
        time_played: chrono::Duration::milliseconds(entry.ms_played),
        track: parse_option(entry.master_metadata_track_name),
        album: parse_option(entry.master_metadata_album_album_name),
        artist: parse_option(entry.master_metadata_album_artist_name),
        id: parse_option(entry.spotify_track_uri),
    })
}

/// Used by [`entry_to_songentry()`]
fn parse_option(opt: Option<String>) -> String {
    match opt {
        Some(data) => data,
        None => String::from("n/a"),
    }
}

/// Used by [`entry_to_songentry()`]
/// for parsing the date from an entry in `endsong.json`
/// and adjusting for time zone and dst
///
/// Relies on [`LOCATION_TZ`]
fn parse_date(ts: &str) -> DateTime<Tz> {
    // timestamp is in "2016-07-21T01:02:07Z" format
    // in UTC!!!!!!!!!
    let ts = DateTime::parse_from_rfc3339(ts).unwrap();
    LOCATION_TZ.from_utc_datetime(&ts.naive_utc())
}
