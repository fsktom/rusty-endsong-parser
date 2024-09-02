//! Module containing representation of a single song stream in endsong.json [`SongEntry`]
//! and [`SongEntries`] which is a collection of [`SongEntry`]s
//!
//! ```
//! use endsong::prelude::*;
//!
//! let paths = vec![format!(
//!     "{}/stuff/example_endsong/endsong_0.json",
//!     std::env::current_dir().unwrap().display()
//! )];
//!
//! let entries = SongEntries::new(&paths)
//!     .unwrap()
//!     .sum_different_capitalization()
//!     .filter(30, TimeDelta::seconds(10));
//! ```

use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use chrono::{DateTime, Local, TimeDelta};
use itertools::Itertools;
use tracing::info;

use crate::aspect;
use crate::find;
use crate::gather;
use crate::parse;

use aspect::{Album, Artist, HasSongs, Music, Song};
use parse::{parse, ParseError};

/// A representation of a single song stream in endsong.json
/// utilized by many functions here.
/// Only for entries which are songs
/// (there are also podcast entries but those are ignored while parsing)
///
/// Contains the relevant metadata of each entry song entry in endsong.json
#[derive(Clone, Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct SongEntry {
    /// the time at which the song has been played
    pub timestamp: DateTime<Local>,
    /// for how long the song has been played
    pub time_played: TimeDelta,
    /// name of the song
    pub track: Rc<str>,
    /// name of the album
    pub album: Rc<str>,
    /// name of the artist
    pub artist: Rc<str>,
    // /// Spotify URI
    // pub id: String,
}
/// Equal if `artist`, `album` and `track` name are the same
impl PartialEq for SongEntry {
    /// Equality for a [`SongEntry`] is when the artist, album, and track name is the same
    fn eq(&self, other: &Self) -> bool {
        // self.id.eq == other.id
        // ^decided not to use that cause it lead to duplicate songs with songs_from_album()
        // sometimes IDs change over time for some songs... thx Spotify :))))
        // that's why equality for a SongEntry is when the artist, album, and track name is the same
        // (also same capitalization!!) -> may change this in future
        self.artist == other.artist && self.album == other.album && self.track == other.track
    }
}
impl Eq for SongEntry {}
/// Hash is the hash of the concatenation of `artist`, `album` and `track`
impl std::hash::Hash for SongEntry {
    /// Hash is the hash of the concatenation of `artist`, `album` and `track`
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let str_to_be_hashed = format!("{}{}{}", self.artist, self.album, self.track);
        str_to_be_hashed.hash(state);
    }
}
/// Ordered by `timestamp`
impl Ord for SongEntry {
    /// Ordered by `timestamp`
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.timestamp.cmp(&other.timestamp)
    }
}
/// Ordered by `timestamp`
impl PartialOrd for SongEntry {
    /// Ordered by `timestamp`
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Struct containing a vector of [`SongEntry`]s and a map of [`Song`]s with their [`TimeDelta`]s
///
/// Fundamental for the use of this program
///
/// It implements [`Deref`][std::ops::Deref], so using `&` will refer to the vector of [`SongEntry`]s.
///
/// ```ignore
/// use endsong::prelude::*;
///
/// let entries = SongEntries::new(&paths)?;
///
/// // .iter() takes in an immutable refrence to the underlying Vec<SongEntry>
/// for entry in entries.iter() {
///     // entry is a &SongEntry
///     println!("{entry:?}");
/// }
///
/// // entries.durations is a HashMap<Song, TimeDelta>
/// let song = Song::new("STYX HELIX", "eYe's", "MYTH & ROID");
/// let duration: TimeDelta = entries.durations.get(&song)?;
/// ```
pub struct SongEntries {
    /// Vector of [`SongEntry`]s
    entries: Vec<SongEntry>,
    /// Map of [`Song`]s with their [durations][TimeDelta]
    pub durations: HashMap<Song, TimeDelta>,
}
impl SongEntries {
    /// Creates an instance of [`SongEntries`]
    ///
    /// # Arguments
    ///
    /// * `paths` - a slice of [`Paths`][`Path`] to each `endsong.json` file.
    ///   Those can be [`Strings`][String], [`strs`][str], [`PathBufs`][std::path::PathBuf]
    ///   or whatever implements [`AsRef<Path>`]
    ///
    /// # Errors
    ///
    /// Will return an error if any of the files can't be opened or read
    pub fn new<P: AsRef<Path> + std::fmt::Debug>(paths: &[P]) -> Result<SongEntries, ParseError> {
        let entries = parse(paths)?;
        let durations = song_durations(&entries);
        Ok(SongEntries { entries, durations })
    }

    /// Sometimes an artist changes the capitalization of their album
    /// or song names. Using this function will change the capitalization
    /// of the album and song names to the most recent ones.
    ///
    /// So that you don't have separate albums listed if they're basically
    /// the same, just with different capitalization.
    ///
    /// E.g. if you have albums called "Fixed" and "FIXED" from the same artist,
    /// it would change all the occurrences of "Fixed" to "FIXED"
    /// (if "FIXED" were the most recent one)
    ///
    /// See [issue #65] for details
    ///
    /// [issue #65]: https://github.com/fsktom/rusty-endsong-parser/issues/65
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn sum_different_capitalization(mut self) -> Self {
        info!("Summing up different capitalization...");

        // 1st: Albums
        // if it's from the same artist and has the same name
        // but different capitalization it's the same album
        let albums = self.iter().map(Album::from).unique().collect_vec();

        // key: lowercase album, value: all album names
        let mut album_versions: HashMap<Album, Vec<Rc<str>>> = HashMap::new();

        for alb in &albums {
            let lowercase_album = Album {
                name: Rc::from(alb.name.to_lowercase()),
                artist: alb.artist.clone(),
            };

            match album_versions.get_mut(&lowercase_album) {
                Some(vec) => vec.push(Rc::clone(&alb.name)),
                None => {
                    album_versions.insert(lowercase_album, vec![Rc::clone(&alb.name)]);
                }
            }
        }

        // the last album in the vector is the one that will be kept
        // cause it's the most recent one
        // key: album, value: newest album name
        let mut album_mappings: HashMap<Album, Rc<str>> = HashMap::with_capacity(albums.len());

        for alb in albums {
            let lowercase_album = Album {
                name: Rc::from(alb.name.to_lowercase()),
                artist: alb.artist.clone(),
            };

            let versions = album_versions.get(&lowercase_album).unwrap();

            // see next loop with if let Some(), you don't need to change
            // anything if there's only one version (no mapping necessary)
            if versions.len() < 2 {
                continue;
            }

            album_mappings.insert(alb, Rc::clone(versions.last().unwrap()));
        }

        for entry in self.iter_mut() {
            let album = Album::from(&*entry);
            if let Some(new_alb) = album_mappings.get(&album) {
                entry.album = Rc::clone(new_alb);
            }
        }

        // 2nd: Songs
        // if it's from the same artist, has the same album and has the same name
        // but different capitalization it's the same song
        // !! doing this after the iteration of changing album names !!
        let songs = self.iter().map(Song::from).unique().collect_vec();

        // key: lowercase song, value: all song names
        let mut song_versions: HashMap<Song, Vec<Rc<str>>> = HashMap::with_capacity(songs.len());

        for song in &songs {
            let lowercase_song = Song {
                name: Rc::from(song.name.to_lowercase()),
                album: song.album.clone(),
            };

            match song_versions.get_mut(&lowercase_song) {
                Some(vec) => vec.push(Rc::clone(&song.name)),
                None => {
                    song_versions.insert(lowercase_song, vec![Rc::clone(&song.name)]);
                }
            }
        }

        // the last song in the vector is the one that will be kept
        // cause it's the most recent one
        // key: song, value: newest song name
        let mut song_mappings: HashMap<Song, Rc<str>> = HashMap::new();

        for song in songs {
            let lowercase_song = Song {
                name: Rc::from(song.name.to_lowercase()),
                album: song.album.clone(),
            };

            let versions = song_versions.get(&lowercase_song).unwrap();

            if versions.len() < 2 {
                continue;
            }

            song_mappings.insert(song, Rc::clone(versions.last().unwrap()));
        }

        for entry in self.iter_mut() {
            let song = Song::from(&*entry);
            if let Some(new_song) = song_mappings.get(&song) {
                entry.track = Rc::clone(new_song);
            }
        }

        // has to be done because some songs change album capitalization
        self.durations = song_durations(&self);

        self
    }

    /// Filters out song entries that have been played
    /// below a certain threshold of their duration
    /// or below a certain absolute [`TimeDelta`]
    ///
    /// # Arguments
    ///
    /// `percent_threshold` - a value between 0 and 100 (%); all songs which have
    /// been played for less than `percent_threshold`% of their duration will be
    /// filtered out; a good default is `30`
    ///
    /// `absolute_threshold` - all songs below this [`TimeDelta`]
    /// will be filtered out; a good default is `TimeDelta::seconds(10)`
    ///
    /// # Panics
    ///
    /// Will panic if `threshhold` is below 0 or above 100
    #[must_use]
    pub fn filter(mut self, percent_threshold: i32, absolute_threshold: TimeDelta) -> Self {
        let length = self.len();
        info!("Filtering out song entries... ({length} song entries before filtering)");
        assert!(
            (0..=100).contains(&percent_threshold),
            "Threshold has to be between 0 and 100"
        );

        // discards every entry whose time_played is below the
        // threshhold percentage of its duration
        self.entries.retain(|entry| {
            // retain is supposed to preserve the order so I don't have to sort again?
            let song = Song::from(entry);
            let duration = *self.durations.get(&song).unwrap();

            entry.time_played >= (duration * percent_threshold) / 100
                && entry.time_played >= absolute_threshold
        });

        info!(
            "{} song entries have been filtered out!",
            length - self.len()
        );

        self
    }

    /// Returns a slice of [`SongEntry`]s between the given dates
    ///
    /// This slice can be used in functions in [`gather`] to gather data between the given dates
    ///
    /// This function uses binary search to find the closest entries to the given dates
    ///
    /// # Panics
    ///
    /// Panics if `start` is after or equal to `end`
    #[must_use]
    pub fn between<'a>(
        &'a self,
        start: &DateTime<Local>,
        end: &DateTime<Local>,
    ) -> &'a [SongEntry] {
        assert!(start <= end, "Start date is after end date!");

        let begin = match self.binary_search_by(|entry| entry.timestamp.cmp(start)) {
            // timestamp from entry
            Ok(i) => i,
            // user inputted date - i because you want it to begin at the closest entry
            Err(i) if i != self.len() => i,
            // user inputted date that's after the last entry
            Err(_) => self.len() - 1,
        };

        let stop = match self.binary_search_by(|entry| entry.timestamp.cmp(end)) {
            // timestamp from entry
            Ok(i) => i,
            // user inputted date - i-1 becuase i would include one entry too much
            Err(i) if i != 0 => i - 1,
            // user inputted date that's before the first entry
            Err(_) => 0,
        };

        &self[begin..=stop]
    }

    /// Returns the date of the first (time-wise) occurrence of any [`SongEntry`]
    ///
    /// # Panics
    ///
    /// Panics if the dataset is empty (but that should never happen)
    #[must_use]
    pub fn first_date(&self) -> DateTime<Local> {
        // bc it's sorted (see parse.rs) -> first entry is the earliest
        self.iter().next().unwrap().timestamp
    }

    /// Returns the date of the last (time-wise) occurrence of any [`SongEntry`]
    ///
    /// # Panics
    ///
    /// Panics if the dataset is empty (but that should never happen)
    #[must_use]
    pub fn last_date(&self) -> DateTime<Local> {
        // bc it's sorted (see parse.rs) -> last entry is the latest
        self.iter().next_back().unwrap().timestamp
    }

    /// Finds the date period with the most listening time for the given `time_span`
    ///
    /// Returns the actual timespan (in case `time_span` was too big or too small)
    /// with the corresponding start and end dates
    ///
    /// Minimum duration is 1 day and maximum duration is the whole dataset, so
    /// a check is performed and the timespan is adjusted accordingly
    ///
    /// # Panics
    ///
    /// Unwraps used on [`TimeDelta::try_days`], but won't panic since
    /// only duration of 1 day created
    #[must_use]
    pub fn max_listening_time(
        &self,
        time_span: TimeDelta,
    ) -> (TimeDelta, DateTime<Local>, DateTime<Local>) {
        let first = self.first_date();
        let last = self.last_date();

        let one_day = TimeDelta::try_days(1).unwrap();

        let actual_time_span = match time_span {
            // maximum duration is whole dataset?
            x if x >= last - first => {
                return (gather::listening_time(self), first, last);
            }
            // minimum duration is 1 day
            x if x < one_day => one_day,
            // duration is within bounds
            _ => time_span,
        };

        let mut highest = TimeDelta::zero();
        let mut start_max = first;
        let mut end_max = first + actual_time_span;

        let mut start = start_max;
        let mut end = end_max;

        while end <= last {
            let current = gather::listening_time(self.between(&start, &end));
            if current > highest {
                highest = current;
                start_max = start;
                end_max = end;
            }
            start += one_day;
            end += one_day;
        }
        (highest, start_max, end_max)
    }

    /// Returns a [`Vec`] with the names of all [`Artists`][Artist] in the dataset
    #[must_use]
    pub fn artists(&self) -> Vec<Rc<str>> {
        self.iter()
            .map(|entry| Rc::clone(&entry.artist))
            .unique()
            .collect_vec()
    }

    /// Returns a [`Vec`] with the names of the [`Albums`][Album]
    /// corresponding to the `artist`
    #[must_use]
    pub fn albums(&self, artist: &Artist) -> Vec<Rc<str>> {
        self.iter()
            .filter(|entry| artist.is_entry(entry))
            .map(|entry| Rc::clone(&entry.album))
            .unique()
            .collect_vec()
    }

    /// Returns a [`Vec`] with the names of the [`Songs`][Song]
    /// corresponding to the `aspect`
    #[must_use]
    pub fn songs<Asp: HasSongs>(&self, aspect: &Asp) -> Vec<Rc<str>> {
        self.iter()
            .filter(|entry| aspect.is_entry(entry))
            .map(|entry| Rc::clone(&entry.track))
            .unique()
            .collect_vec()
    }

    /// Counts up the plays of all [`Music`] in a collection
    #[must_use]
    pub fn gather_plays_of_many<Asp: Music>(&self, aspects: &[Asp]) -> usize {
        gather::plays_of_many(self, aspects)
    }

    /// Adds search capability
    ///
    /// Use with methods from [`Find`] to search for valid artist, album or song
    /// names from the dataset and more.
    #[must_use]
    pub fn find(&self) -> Find {
        Find(self)
    }
}
// https://users.rust-lang.org/t/how-can-i-return-reference-of-the-struct-field/36325/2
// so that when you use &self it refers to &self.0 (Vec<SongEntry>)
impl std::ops::Deref for SongEntries {
    type Target = Vec<SongEntry>;
    fn deref(&self) -> &Vec<SongEntry> {
        &self.entries
    }
}
impl std::ops::DerefMut for SongEntries {
    fn deref_mut(&mut self) -> &mut Vec<SongEntry> {
        &mut self.entries
    }
}
// TryFrom because of ergonomic API design -> into() etc.
// see https://youtu.be/0zOg8_B71gE?t=922
impl<P: AsRef<Path> + std::fmt::Debug> TryFrom<&[P]> for SongEntries {
    type Error = ParseError;

    /// Creates an instance of [`SongEntries`] from a slice of [`Path`][`Path`]s
    ///
    /// Those can be [`Strings`][String], [`strs`][str], [`PathBufs`][std::path::PathBuf] or whatever implements [`AsRef<Path>`]
    fn try_from(path: &[P]) -> Result<Self, Self::Error> {
        SongEntries::new(path)
    }
}

/// Returns a [`HashMap`] with the [`Songs`][Song] as keys and
/// their [durations][TimeDelta]s as values
///
/// A duration is calculated by finding the most common playtime of a song.
/// If multiple playtimes have the same amount of plays, the larger/longer
/// one is chosen.
fn song_durations(entries: &[SongEntry]) -> HashMap<Song, TimeDelta> {
    info!("Calculating song durations...");

    // first, calculate the amount of plays each song playtime has
    let duration_counts: HashMap<(Song, TimeDelta), usize> = entries
        .iter()
        .map(|e| (Song::from(e), e.time_played))
        .counts();

    // 10k is just a guess for amount of unique songs
    let mut song_durations: HashMap<Song, TimeDelta> = HashMap::with_capacity(10_000);

    for ((song, duration), plays) in &duration_counts {
        song_durations
            .entry(song.clone())
            .and_modify(|current_duration| {
                let current_plays = duration_counts[&(song.clone(), *current_duration)];

                // because the longest duration is not necessarily the correct one
                // e.g. if you skip through the song `ms_played`
                // will be longer than the actual song length
                // so we take the most common duration
                let more_common = current_plays < *plays;

                // but multiple durations can have the same maximum occurrence
                // so we then take the longest maximum duration
                let same_but_longer = current_plays == *plays && *current_duration < *duration;

                if more_common || same_but_longer {
                    *current_duration = *duration;
                }
            })
            .or_insert(*duration);
    }
    song_durations
}

/// Used by [`SongEntries`] as a wrapper for [`find`] methods
///
/// Created with [`SongEntries::find`]
pub struct Find<'a>(&'a SongEntries);
impl<'a> Find<'a> {
    /// Searches the entries for possible artists
    ///
    /// Case-insensitive and returns the [`Artist`] with proper capitalization.
    /// The vector contains multiple [`Artist`]s if they're called the same,
    /// but their names are capitalized diffferently
    ///
    /// Vector is guaranteed to be non-empty if [`Some`]
    ///
    /// See #2 <https://github.com/fsktom/rusty-endsong-parser/issues/2>
    #[must_use]
    pub fn artist(&self, artist_name: &str) -> Option<Vec<Artist>> {
        find::artist(self.0, artist_name)
    }

    /// Searches the entries for possible albums
    ///
    /// Case-insensitive and returns the [`Album`] with proper capitalization
    /// The vector contains multiple [`Album`]s if they're called the same,
    /// but their names are capitalized diffferently
    /// (Guaranteed for there to be only one version if you use
    /// [`SongEntries::sum_different_capitalization`][crate::entry::SongEntries::sum_different_capitalization])
    ///
    /// Vector is guaranteed to be non-empty if [`Some`]
    ///
    /// See #2 <https://github.com/fsktom/rusty-endsong-parser/issues/2>
    #[must_use]
    pub fn album(&self, album_name: &str, artist_name: &str) -> Option<Vec<Album>> {
        find::album(self.0, album_name, artist_name)
    }

    /// Searches the entries possible songs (in that specific album)
    /// in the dataset
    ///
    /// Case-insensitive and returns the [`Song`] with proper capitalization
    /// The vector contains multiple [`Song`]s if they're called the same,
    /// but their names are capitalized diffferently
    /// (Guaranteed for there to be only one version if you use
    /// [`SongEntries::sum_different_capitalization`][crate::entry::SongEntries::sum_different_capitalization])
    ///
    /// Vector is guaranteed to be non-empty if [`Some`]
    ///
    /// See #2 <https://github.com/fsktom/rusty-endsong-parser/issues/2>
    #[must_use]
    pub fn song_from_album(
        &self,
        song_name: &str,
        album_name: &str,
        artist_name: &str,
    ) -> Option<Vec<Song>> {
        find::song_from_album(self.0, song_name, album_name, artist_name)
    }

    /// Searches the dataset for multiple versions of a song
    ///
    /// Case-insensitive and returns a [`Vec<Song>`] containing an instance
    /// of [`Song`] for every album it's been found in with proper capitalization
    ///
    /// See #2 <https://github.com/fsktom/rusty-endsong-parser/issues/2>
    #[must_use]
    pub fn song(&self, song_name: &str, artist_name: &str) -> Option<Vec<Song>> {
        find::song(self.0, song_name, artist_name)
    }
}
