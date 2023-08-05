//! Module responsible for deserializing the endsong.json files
//! into usable Rust data types

use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::rc::Rc;

use chrono::{DateTime, Duration, TimeZone};
use chrono_tz::Tz;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::entry::SongEntry;

/// responsible for time zone handling
///
/// see issue #4 <https://github.com/fsktom/rusty-endsong-parser/issues/4>
///
/// used for parsing the timestamp from `endsong.json` relative to the user's time zone
///
/// Currently hard-coded to Europe/Berlin
pub const LOCATION_TZ: Tz = chrono_tz::Europe::Berlin;

// https://stackoverflow.com/questions/44205435/how-to-deserialize-a-json-file-which-contains-null-values-using-serde
// null values are either skipped (defaulted to unit tuple or are an Option)
/// General/raw struct for a single entry in endsong.json
/// (which are an array of such structs)
///
/// Raw because it's directly the deserialization from endsong.json
///
/// These are later "converted" to [`SongEntry`] if they represent a song stream.
/// Podcast streams are ignored.
#[derive(Serialize, Deserialize, Debug)]
struct Entry {
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
    master_metadata_track_name: Option<Rc<str>>,
    /// Name of the artist
    ///
    /// Option because the field will be empty if it's a podcast
    master_metadata_album_artist_name: Option<Rc<str>>,
    /// Name of the album
    ///
    /// Option because the field will be empty if it's a podcast
    master_metadata_album_album_name: Option<Rc<str>>,
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

/// Main parsing function that parses many `endsong.json` files
///
/// Returns a vector of [`SongEntry`]s sorted by timestamp
///
/// # Errors
///
/// Will return an error if any of the files can't be opened or read
pub fn parse<P: AsRef<Path>>(paths: &[P]) -> Result<Vec<SongEntry>, Box<dyn Error>> {
    // at least for me: about 15.8k-15.95k entries per file
    // to prevent reallocations?
    let mut song_entries: Vec<SongEntry> = Vec::with_capacity(16_000 * paths.len());
    for path in paths {
        let mut one = parse_single(path)?;
        song_entries.append(&mut one);
    }

    // sort by timestamp (oldest streams are first, newest are last)
    song_entries.sort_unstable_by(|a, b| a.timestamp.cmp(&b.timestamp));

    Ok(song_entries)
}

/// Responsible for parsing the a single `endsong.json` file into a vector of [`SongEntry`]
fn parse_single<P: AsRef<Path>>(path: P) -> Result<Vec<SongEntry>, Box<dyn Error>> {
    // https://github.com/serde-rs/json/issues/160#issuecomment-253446892
    let mut file_contents = String::new();
    File::open(path)?.read_to_string(&mut file_contents)?;
    let full_entries: Vec<Entry> = serde_json::from_str(&file_contents)?;

    // convert each Entry to a SongEntry (ignoring podcast streams)
    let song_entries = full_entries
        .into_iter()
        .filter_map(entry_to_songentry)
        .collect_vec();

    Ok(song_entries)
}

/// Converts the genral [`Entry`] to a more specific [`SongEntry`]
fn entry_to_songentry(entry: Entry) -> Option<SongEntry> {
    // to remove podcast entries
    // if the track is None, so are album and artist
    entry.master_metadata_track_name.as_ref()?;

    Some(SongEntry {
        timestamp: parse_date(&entry.ts),
        time_played: Duration::milliseconds(entry.ms_played),
        // unwrap() ok because we already checked for track_name above
        // if trackname isn't null then these fields aren't either
        track: entry.master_metadata_track_name.unwrap(),
        album: entry.master_metadata_album_album_name.unwrap(),
        artist: entry.master_metadata_album_artist_name.unwrap(),
        id: entry.spotify_track_uri.unwrap(),
    })
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
