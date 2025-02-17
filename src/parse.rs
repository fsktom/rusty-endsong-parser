//! Module responsible for deserializing the endsong.json files
//! into usable Rust data types

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use tracing::instrument;

use chrono::{DateTime, Local, TimeDelta, TimeZone};
use itertools::Itertools;
use serde::Deserialize;
use thiserror::Error;
use tracing::{error, info, info_span};

use crate::entry::SongEntry;

/// Errors that can occur when parsing an endsong.json file
#[derive(Error, Debug)]
enum SingleParseError {
    /// Used when serde deserialization fails
    #[error("Error while parsing the file: {0}")]
    Serde(#[from] serde_json::Error),
    /// Used when reading the file fails
    #[error("Error while opening the file: {0}")]
    Io(#[from] std::io::Error),
}

/// Errors that can occur when parsing the endsong.json files
#[derive(Error, Debug)]
pub enum ParseError {
    /// Used when serde deserialization fails
    #[error("Error while parsing {1}: {0}")]
    Serde(serde_json::Error, Box<Path>),
    /// Used when reading the file fails
    #[error("Error while opening {1}: {0}")]
    Io(std::io::Error, Box<Path>),
}

// https://stackoverflow.com/questions/44205435/how-to-deserialize-a-json-file-which-contains-null-values-using-serde
// null values are either skipped (defaulted to unit tuple or are an Option)
/// General/raw struct for a single entry in endsong.json
/// (which are an array of such structs)
///
/// Raw because it's directly the deserialization from endsong.json
///
/// These are later "converted" to [`SongEntry`] if they represent a song stream.
/// Podcast streams are ignored.
#[derive(Deserialize, Debug, Clone)]
struct Entry {
    /// timestamp in `"YYY-MM-DD 13:30:30"` format
    ts: String,
    /// Skipped
    #[serde(skip_deserializing)]
    _username: (),
    /// Skipped for now: maybe use it for sth
    #[serde(skip_deserializing)]
    _platform: String,
    /// Miliseconds the song has been played for
    ms_played: i64,
    /// Skipped
    #[serde(skip_deserializing)]
    _conn_country: (),
    /// Skipped
    #[serde(skip_deserializing)]
    _ip_addr_decrypted: (),
    /// Skipped
    #[serde(skip_deserializing)]
    _user_agent_decrypted: (),
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
    #[serde(skip_deserializing)]
    _spotify_track_uri: Option<String>,
    /// TBD: Podcast stuff
    #[serde(skip_deserializing)]
    _episode_name: (),
    /// TBD: Podcast stuff
    #[serde(skip_deserializing)]
    /// TBD: Podcast stuff
    _episode_show_name: (),
    #[serde(skip_deserializing)]
    /// TBD: Podcast stuff
    _spotify_episode_uri: (),
    /// Skipped for now: maybe use it for sth
    #[serde(skip_deserializing)]
    _reason_start: String,
    /// Skipped for now: maybe use it for sth
    #[serde(skip_deserializing)]
    _reason_end: String,
    /// Skipped for now: maybe use it for sth
    #[serde(skip_deserializing)]
    _shuffle: bool,
    /// Skipped for now: maybe use it for sth
    #[serde(skip_deserializing)]
    _skipped: Option<bool>,
    /// Skipped
    #[serde(skip_deserializing)]
    _offline: (),
    /// Skipped
    #[serde(skip_deserializing)]
    _offline_timestamp: (),
    /// Skipped
    #[serde(skip_deserializing)]
    _incognito_mode: (),
}

/// Main parsing function that parses many `endsong.json` files
///
/// Returns a vector of [`SongEntry`]s sorted by timestamp
///
/// # Errors
///
/// Will return an error if any of the files can't be opened or read
pub fn parse<P: AsRef<Path> + std::fmt::Debug>(paths: &[P]) -> Result<Vec<SongEntry>, ParseError> {
    info!("Parsing {} files", paths.len());
    // at least for me: about 15.8k-15.95k entries per file
    // to prevent reallocations?
    let mut song_entries: Vec<SongEntry> = Vec::with_capacity(16_000 * paths.len());

    let mut song_names: HashMap<String, Arc<str>> = HashMap::with_capacity(10_000);
    let mut album_names: HashMap<String, Arc<str>> = HashMap::with_capacity(10_000);
    let mut artist_names: HashMap<String, Arc<str>> = HashMap::with_capacity(5_000);

    let mut timestamps: HashSet<DateTime<Local>> = HashSet::with_capacity(16_000 * paths.len());

    for path in paths {
        let p = path.as_ref();
        let span = info_span!("file", path = ?p);
        let _guard = span.enter();
        info!("currently parsing");
        let mut one = match parse_single(
            path,
            &mut song_names,
            &mut album_names,
            &mut artist_names,
            &mut timestamps,
        ) {
            Ok(parsed) => parsed,
            Err(SingleParseError::Io(e)) => {
                error!("failed to open {p:?}");
                return Err(ParseError::Io(e, p.into()));
            }
            Err(SingleParseError::Serde(e)) => {
                error!("failed to parse {p:?}");
                return Err(ParseError::Serde(e, p.into()));
            }
        };
        song_entries.append(&mut one);
    }

    // newer endsong files should already be sorted by timestamp
    // (oldest streams are first, newest are last)
    // sorting if you're using older (pre-2023) files
    if !song_entries.is_sorted() {
        song_entries.sort_unstable();
    }

    Ok(song_entries)
}

/// Responsible for parsing the a single `endsong.json` file into a vector of [`SongEntry`]
#[instrument]
fn parse_single<P: AsRef<Path> + std::fmt::Debug>(
    path: P,
    song_names: &mut HashMap<String, Arc<str>>,
    album_names: &mut HashMap<String, Arc<str>>,
    artist_names: &mut HashMap<String, Arc<str>>,
    timestamps: &mut HashSet<DateTime<Local>>,
) -> Result<Vec<SongEntry>, SingleParseError> {
    // https://github.com/serde-rs/json/issues/160#issuecomment-253446892
    let mut file_contents = String::new();
    File::open(path)?.read_to_string(&mut file_contents)?;
    let full_entries: Vec<Entry> = serde_json::from_str(&file_contents)?;

    // convert each Entry to a SongEntry (ignoring podcast streams)
    let song_entries = full_entries
        .into_iter()
        .filter_map(|entry| {
            entry_to_songentry(entry, song_names, album_names, artist_names, timestamps)
        })
        .collect_vec();

    Ok(song_entries)
}

/// Converts the genral [`Entry`] to a more specific [`SongEntry`]
fn entry_to_songentry(
    entry: Entry,
    song_names: &mut HashMap<String, Arc<str>>,
    album_names: &mut HashMap<String, Arc<str>>,
    artist_names: &mut HashMap<String, Arc<str>>,
    timestamps: &mut HashSet<DateTime<Local>>,
) -> Option<SongEntry> {
    let timestamp = parse_date(&entry.ts);
    // to remove entries with duplicate timestamps
    // (bc Spotify is stupid sometimes)
    if !timestamps.insert(timestamp) {
        return None;
    }

    // ? to remove podcast entries
    // if the track is None, so are album and artist

    let track = map_arc_name(song_names, &entry.master_metadata_track_name?);
    let album = map_arc_name(album_names, &entry.master_metadata_album_album_name?);
    let artist = map_arc_name(artist_names, &entry.master_metadata_album_artist_name?);

    Some(SongEntry {
        timestamp,
        // unwrap fine since ms_played will never be big enough...
        time_played: TimeDelta::try_milliseconds(entry.ms_played).unwrap(),
        track,
        album,
        artist,
        // id: entry.spotify_track_uri?,
    })
}

/// Checks if the given `name` is in the `map` and does [`Arc::clone`] on it
///
/// If it's not in the map, it clones the String value into an
/// [`Arc`] and inserts it into the map
fn map_arc_name(map: &mut HashMap<String, Arc<str>>, name: &str) -> Arc<str> {
    if let Some(name_rc) = map.get(name) {
        Arc::clone(name_rc)
    } else {
        map.insert(name.to_string(), Arc::from(name));
        Arc::clone(map.get(name).unwrap())
    }
}

/// Used by [`entry_to_songentry()`]
/// for parsing the date from an entry in `endsong.json`
/// and adjusting for local time zone and dst
fn parse_date(ts: &str) -> DateTime<Local> {
    // timestamp is in "2016-07-21T01:02:07Z" format
    // in UTC!!!!!!!!!
    let ts = DateTime::parse_from_rfc3339(ts).unwrap();
    Local.from_utc_datetime(&ts.naive_utc())
}
