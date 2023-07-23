//! Module containg many types used throughout the program
// https://doc.rust-lang.org/stable/book/ch06-01-defining-an-enum.html
pub use plotly::Trace;

use std::cmp::Ordering;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::path::Path;

use chrono::{DateTime, Duration};
use chrono_tz::Tz;
use itertools::Itertools;

use crate::find;
use crate::gather;
use crate::parse;
use crate::plot;

/// Used for functions that accept either
/// a [`Song`], [`Album`] or [`Artist`] struct
pub trait Music: Display + Clone + Eq + Ord {
    /// Checks if a [`SongEntry`] is a [`Music`]
    fn is_entry(&self, entry: &SongEntry) -> bool;

    /// Checks if a [`SongEntry`] is a [`Music`] but case insensitive
    ///
    /// Performs `.to_lowercase()` ONLY on `entry`, NOT on [`self`].
    /// Make sure in advance that [`self`] fields are lowercase.
    fn is_entry_lowercase(&self, entry: &SongEntry) -> bool;
}

/// Trait used to accept only [`Artist`] and [`Album`]
pub trait HasSongs: Music {}

/// Trait used to accept only [`Album`] and [`Song`]
pub trait HasArtist: Music {
    /// Returns a reference to the corresponding [`Artist`]
    fn artist(&self) -> &Artist;
}

/// Struct for representing an artist
#[derive(PartialEq, Eq, Hash, Debug, Clone, PartialOrd, Ord)]
pub struct Artist {
    /// Name of the artist
    pub name: String,
}
impl Artist {
    /// Creates an instance of Artist
    pub fn new<S: Into<String>>(artist_name: S) -> Artist {
        Artist {
            name: artist_name.into(),
        }
    }
}
impl Display for Artist {
    /// Formats the struct in "<`artist_name`>" format
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
impl From<&SongEntry> for Artist {
    fn from(entry: &SongEntry) -> Self {
        Artist::new(&entry.artist)
    }
}
impl Music for Artist {
    fn is_entry(&self, entry: &SongEntry) -> bool {
        entry.artist == self.name
    }
    fn is_entry_lowercase(&self, entry: &SongEntry) -> bool {
        entry.artist.to_lowercase() == self.name
    }
}
impl HasSongs for Artist {}

/// Struct for representing an album
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct Album {
    /// Name of the album
    pub name: String,
    /// Artist of the album
    pub artist: Artist,
}
impl Album {
    /// Creates an instance of Album
    pub fn new<S: Into<String>>(album_name: S, artist_name: S) -> Album {
        Album {
            name: album_name.into(),
            artist: Artist::new(artist_name),
        }
    }
}
impl Display for Album {
    /// Formats the struct in "<`artist_name`> - <`album_name`>" format
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.artist.name, self.name)
    }
}
impl PartialOrd for Album {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.artist.partial_cmp(&other.artist) {
            // if the artists are the same, compare the albums
            Some(Ordering::Equal) => self.name.partial_cmp(&other.name),
            // otherwise, compare the artists
            _ => self.artist.partial_cmp(&other.artist),
        }
    }
}
impl Ord for Album {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.artist.cmp(&other.artist) {
            // if the artists are the same, compare the albums
            Ordering::Equal => self.name.cmp(&other.name),
            // otherwise, compare the artists
            _ => self.artist.cmp(&other.artist),
        }
    }
}
impl From<&SongEntry> for Album {
    fn from(entry: &SongEntry) -> Self {
        Album::new(&entry.album, &entry.artist)
    }
}
impl Music for Album {
    fn is_entry(&self, entry: &SongEntry) -> bool {
        entry.artist == self.artist.name && entry.album == self.name
    }
    fn is_entry_lowercase(&self, entry: &SongEntry) -> bool {
        entry.artist.to_lowercase() == self.artist.name && entry.album.to_lowercase() == self.name
    }
}
impl HasSongs for Album {}
impl HasArtist for Album {
    fn artist(&self) -> &Artist {
        &self.artist
    }
}

/// Struct for representing a song
// to allow for custom HashMap key
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct Song {
    /// Name of the song
    pub name: String,
    /// The album this song is from
    pub album: Album,
    // pub id: String,
}
impl Song {
    /// Creates an instance of Song
    pub fn new<S: Into<String>>(song_name: S, album_name: S, artist_name: S) -> Song {
        Song {
            name: song_name.into(),
            album: Album::new(album_name, artist_name),
        }
    }
}
impl Display for Song {
    /// Formats the struct in "<`artist_name`> - <`song_name`> (<`album_name`>)" format
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} - {} ({})",
            self.album.artist.name, self.name, self.album.name
        )
    }
}
impl PartialOrd for Song {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.album.artist.partial_cmp(&other.album.artist) {
            // if the artists are the same, compare the song names
            Some(Ordering::Equal) => match self.name.partial_cmp(&other.name) {
                // if the song names are the same, compare the album names
                Some(Ordering::Equal) => self.album.name.partial_cmp(&other.album.name),
                // otherwise, compare the song names
                _ => self.name.partial_cmp(&other.name),
            },
            // otherwise, compare the artists
            _ => self.album.artist.partial_cmp(&other.album.artist),
        }
    }
}
impl Ord for Song {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.album.artist.cmp(&other.album.artist) {
            // if the artists are the same, compare the song names
            Ordering::Equal => match self.name.cmp(&other.name) {
                // if the song names are the same, compare the album names
                Ordering::Equal => self.album.name.cmp(&other.album.name),
                // otherwise, compare the song names
                _ => self.name.cmp(&other.name),
            },
            // otherwise, compare the artists
            _ => self.album.artist.cmp(&other.album.artist),
        }
    }
}
impl From<&SongEntry> for Song {
    fn from(entry: &SongEntry) -> Self {
        Song::new(&entry.track, &entry.album, &entry.artist)
    }
}
impl Music for Song {
    fn is_entry(&self, entry: &SongEntry) -> bool {
        entry.artist == self.album.artist.name
            && entry.album == self.album.name
            && entry.track == self.name
    }
    fn is_entry_lowercase(&self, entry: &SongEntry) -> bool {
        entry.artist.to_lowercase() == self.album.artist.name
            && entry.album.to_lowercase() == self.album.name
            && entry.track.to_lowercase() == self.name
    }
}
impl HasArtist for Song {
    fn artist(&self) -> &Artist {
        &self.album.artist
    }
}

/// A more specific version of [`parse::Entry`]
/// utilized by many functions here.
/// Only for entries which are songs (there are also podcast entries)
///
/// Contains the relevant metadata of each entry song entry in endsong.json
#[derive(Clone, Debug)]
pub struct SongEntry {
    /// the time at which the song has been played
    pub timestamp: DateTime<Tz>,
    /// for how long the song has been played
    pub time_played: Duration,
    /// name of the song
    pub track: String,
    /// name of the album
    pub album: String,
    /// name of the artist
    pub artist: String,
    /// Spotify URI
    pub id: String,
}
impl PartialEq for SongEntry {
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
impl std::hash::Hash for SongEntry {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let str_to_be_hashed = format!("{}{}{}", self.artist, self.album, self.track);
        str_to_be_hashed.hash(state);
    }
}

/// Struct containing a vector of [`SongEntry`]
///
/// Fundamental for the use of this program
pub struct SongEntries(Vec<SongEntry>);
impl SongEntries {
    /// Creates an instance of [`SongEntries`]
    ///
    /// Returns an [`Error`] if it encounters problems while parsing
    ///
    /// * `paths` - a slice of [`Path`][`Path`]s to each `endsong.json` file
    pub fn new<P: AsRef<Path>>(paths: &[P]) -> Result<SongEntries, Box<dyn Error>> {
        Ok(SongEntries(parse::parse(paths)?))
    }

    /// Returns the date of the first (time-wise) occurrence of any [`SongEntry`]
    pub fn first_date(&self) -> DateTime<Tz> {
        // bc it's sorted (see parse.rs) -> first entry is the earliest
        self.iter().next().unwrap().timestamp
    }

    /// Returns the date of the last (time-wise) occurrence of any [`SongEntry`]
    pub fn last_date(&self) -> DateTime<Tz> {
        // bc it's sorted (see parse.rs) -> last entry is the latest
        self.iter().next_back().unwrap().timestamp
    }

    /// Returns the total time listened
    pub fn listening_time(&self) -> Duration {
        gather::listening_time(self)
    }

    /// Returns the time listened in a given date period
    ///
    /// # Panics
    ///
    /// Panics if `start` is after or equal to `end`
    pub fn listening_time_date(&self, start: &DateTime<Tz>, end: &DateTime<Tz>) -> Duration {
        gather::listening_time_date(self, start, end)
    }

    /// Finds the date period with the most listening time for the given `time_span`
    pub fn max_listening_time(
        &self,
        time_span: Duration,
    ) -> (Duration, DateTime<Tz>, DateTime<Tz>) {
        let first = self.first_date();
        let last = self.last_date();

        let actual_time_span = match time_span {
            // maximum duration is whole dataset?
            x if x >= last - first => {
                return (self.listening_time(), first, last);
            }
            // minimum duration is 1 day
            x if x < Duration::days(1) => Duration::days(1),
            //
            _ => time_span,
        };

        let mut highest = Duration::seconds(0);
        let mut start_max = first;
        let mut end_max = first + actual_time_span;

        let mut start = start_max;
        let mut end = end_max;

        while end <= last {
            let current = self.listening_time_date(&start, &end);
            if current > highest {
                highest = current;
                start_max = start;
                end_max = end;
            }
            start += Duration::days(1);
            end += Duration::days(1);
        }
        (highest, start_max, end_max)
    }

    /// Returns a [`Vec<String>`] with the names of all [`Artists`][Artist] in the dataset
    pub fn artists(&self) -> Vec<String> {
        self.iter()
            .map(|entry| entry.artist.clone())
            .unique()
            .collect::<Vec<String>>()
    }

    /// Returns a [`Vec<String>`] with the names of the [`Albums`][Album]
    /// corresponding to the `artist`
    pub fn albums(&self, artist: &Artist) -> Vec<String> {
        self.iter()
            .filter(|entry| artist.is_entry(entry))
            .map(|entry| entry.album.clone())
            .unique()
            .collect::<Vec<String>>()
    }

    /// Returns a [`Vec<String>`] with the names of the [`Songs`][Song]
    /// corresponding to the `aspect`
    pub fn songs<Asp: HasSongs>(&self, aspect: &Asp) -> Vec<String> {
        self.iter()
            .filter(|entry| aspect.is_entry(entry))
            .map(|entry| entry.track.clone())
            .unique()
            .collect::<Vec<String>>()
    }

    /// Returns the length of the song
    ///
    /// `song` has to be a valid entry in the dataset
    ///
    /// Assuming the length of the song is the most common playime of a song.
    /// Not the highest, because skipping through a song while playing it can
    /// make the `ms_played` value of the entry higher than the actual
    /// length of the song.
    ///
    /// # Panics
    ///
    /// Panics if `song` is not a valid entry in the dataset
    pub fn song_length(&self, song: &Song) -> Duration {
        // map of durations with their amount of occurences
        let mut durations = HashMap::<Duration, usize>::with_capacity(10);

        for dur in self
            .iter()
            .filter(|entry| song.is_entry(entry))
            .map(|entry| entry.time_played)
        {
            durations
                .entry(dur)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }

        if durations.len() == 1 {
            return *durations.keys().next().unwrap();
        }

        // has to be done because possible that multiple durations
        // have the same amount of occurences
        // -> from max_by_key docs:
        // "If several elements are equally maximum, the last element is returned"
        // but I need the one with the max occurrence AND the max duration
        // otherwise it's not deterministic
        // notice the non-determinism while doing SongEntries.filter() xd
        let max_occurrence = *durations.iter().max_by_key(|(_, count)| *count).unwrap().1;

        durations
            .into_iter()
            .filter(|(_, count)| *count == max_occurrence)
            .max_by_key(|(dur, _)| *dur)
            // unwrap() ok because assumption is that `song` exists in dataset
            .unwrap()
            .0
    }

    /// Counts up the plays of all [`Music`] in a collection
    pub fn gather_plays_of_many<Asp: Music>(&self, aspects: &[Asp]) -> usize {
        gather::plays_of_many(self, aspects)
    }

    /// Counts up the plays of all [`Music`] in a collection within the date range
    ///
    /// # Panics
    ///
    /// Panics if `start` is after or equal to `end`
    pub fn gather_plays_of_many_date<Asp: Music>(
        &self,
        aspects: &[Asp],
        start: &DateTime<Tz>,
        end: &DateTime<Tz>,
    ) -> usize {
        gather::plays_of_many_date(self, aspects, start, end)
    }

    /// Adds search capability
    ///
    /// Use with methods from [`Find`]: [`.artist()`][Find::artist()], [`.album()`][Find::album()],
    /// [`.song_from_album()`][Find::song_from_album()] and [`.song()`][Find::song()]
    pub fn find(&self) -> Find {
        Find(self)
    }

    /// Used to get traces for [`plot_single()`] and [`plot_compare()`]
    pub fn traces(&self) -> Traces {
        Traces(self)
    }

    /// Returns a [`HashMap`] with the [`Songs`][Song] as keys and
    /// their [`Durations`][Duration] as values
    pub fn song_durations(&self) -> HashMap<Song, Duration> {
        let mut big_boy = HashMap::<Song, HashMap<Duration, usize>>::with_capacity(10_000);

        for entry in self.iter() {
            let song = Song::from(entry);
            let dur = entry.time_played;

            // see .song_length() for explanation
            if let Some(durations) = big_boy.get_mut(&song) {
                durations
                    .entry(dur)
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            } else {
                let mut durations = HashMap::<Duration, usize>::with_capacity(10);
                durations.insert(dur, 1);
                big_boy.insert(song, durations);
            }
        }

        big_boy
            .iter()
            .map(|(song, durations)| {
                let max_occurrence = durations.iter().max_by_key(|(_, count)| *count).unwrap().1;

                let dur = durations
                    .iter()
                    .filter(|(_, count)| *count == max_occurrence)
                    .max_by_key(|(dur, _)| *dur)
                    .unwrap()
                    .0;

                (song.clone(), *dur)
            })
            .collect()
    }

    /// Filters out song entries that have been played
    /// below a certain threshold of their duration
    ///
    /// `threshold` is a value between 0 and 100 (%)
    #[allow(dead_code)]
    pub fn filter(&mut self, threshold: i32) {
        assert!(
            (0..=100).contains(&threshold),
            "Threshold has to be between 0 and 100"
        );

        let durations = self.song_durations();

        // discards every entry whose time_played is below the
        // threshhold percentage of its duration
        self.retain(|entry| {
            // retain is supposed to preserve the order so I don't have to sort again?
            let (_, dur) = durations
                .iter()
                .find(|(son, _)| son.is_entry(entry))
                .unwrap();

            entry.time_played >= (*dur * threshold) / 100
        });
    }
}
// https://users.rust-lang.org/t/how-can-i-return-reference-of-the-struct-field/36325/2
// so that when you use &self it refers to &self.0 (Vec<SongEntry>)
impl std::ops::Deref for SongEntries {
    type Target = Vec<SongEntry>;
    fn deref(&self) -> &Vec<SongEntry> {
        &self.0
    }
}
impl std::ops::DerefMut for SongEntries {
    fn deref_mut(&mut self) -> &mut Vec<SongEntry> {
        &mut self.0
    }
}
// TryFrom because of ergonomic API design -> into() etc.
// see https://youtu.be/0zOg8_B71gE?t=922
impl<P: AsRef<Path>> TryFrom<&[P]> for SongEntries {
    type Error = Box<dyn Error>;

    /// Creates an instance of [`SongEntries`] from a slice of [`Path`][`Path`]s
    ///
    /// Those can be [`Strings`][String], [`strs`][str], [`PathBufs`][std::path::PathBuf] or whatever implements [`AsRef<Path>`]
    fn try_from(path: &[P]) -> Result<Self, Self::Error> {
        SongEntries::new(path)
    }
}

/// Used by [`SongEntries`] as a wrapper for [`find`] methods
pub struct Find<'a>(&'a SongEntries);
impl<'a> Find<'a> {
    /// Searches the entries for if the given artist exists in the dataset
    ///
    /// Case-insensitive and returns the [`Artist`] with proper capitalization
    /// (i.e. the capitalization of the first entry it finds)
    ///
    /// See #2 <https://github.com/fsktom/rusty-endsong-parser/issues/2>
    pub fn artist(&self, artist_name: &str) -> Option<Artist> {
        find::artist(self.0, artist_name)
    }

    /// Searches the entries for if the given album exists in the dataset
    ///
    /// Case-insensitive and returns the [`Album`] with proper capitalization
    /// (i.e. the capitalization of the first entry it finds)
    ///
    /// See #2 <https://github.com/fsktom/rusty-endsong-parser/issues/2>
    pub fn album(&self, album_name: &str, artist_name: &str) -> Option<Album> {
        find::album(self.0, album_name, artist_name)
    }

    /// Searches the entries for if the given song (in that specific album)
    /// exists in the dataset
    ///
    /// Case-insensitive and returns the [`Song`] with proper capitalization
    /// (i.e. the capitalization of the first entry it finds)
    ///
    /// See #2 <https://github.com/fsktom/rusty-endsong-parser/issues/2>
    pub fn song_from_album(
        &self,
        song_name: &str,
        album_name: &str,
        artist_name: &str,
    ) -> Option<Song> {
        find::song_from_album(self.0, song_name, album_name, artist_name)
    }

    /// Searches the dataset for multiple versions of a song
    ///
    /// Case-insensitive and returns a [`Vec<Song>`] containing an instance
    /// of [`Song`] for every album it's been found in with proper capitalization
    ///
    /// See #2 <https://github.com/fsktom/rusty-endsong-parser/issues/2>
    pub fn song(&self, song_name: &str, artist_name: &str) -> Option<Vec<Song>> {
        find::song(self.0, song_name, artist_name)
    }

    /// Returns a [`Vec<Song>`] with all the songs in the given album
    ///
    /// # Panics
    ///
    /// Panics if `album` is not in the dataset
    pub fn songs_from_album(&self, album: &Album) -> Vec<Song> {
        find::songs_from_album(self.0, album)
    }
}

/// Used by [`SongEntries`] to get traces for plots in
/// [`plot_single()`] and [`plot_compare()`]
pub struct Traces<'a>(&'a SongEntries);
impl<'a> Traces<'a> {
    /// Returns a trace of the absolute plays of an `aspect`
    ///
    /// Wrapper for [`plot::absolute::aspect()`]
    pub fn absolute<Asp: Music>(&self, aspect: &Asp) -> (Box<dyn Trace>, String) {
        plot::absolute::aspect(self.0, aspect)
    }

    /// Returns a trace of the plays relative to all plays
    ///
    /// Wrapper for [`plot::relative::to_all()`]
    pub fn relative<Asp: Music>(&self, aspect: &Asp) -> (Box<dyn Trace>, String) {
        plot::relative::to_all(self.0, aspect)
    }

    /// Returns a trace of the plays relative to the artist
    ///
    /// Wrapper for [`plot::relative::to_artist()`]
    pub fn relative_to_artist<Asp: HasArtist>(&self, aspect: &Asp) -> (Box<dyn Trace>, String) {
        plot::relative::to_artist(self.0, aspect)
    }

    /// Returns a trace of the plays relative to the album
    ///
    /// Wrapper for [`plot::relative::to_album()`]
    pub fn relative_to_album(&self, song: &Song) -> (Box<dyn Trace>, String) {
        plot::relative::to_album(self.0, song)
    }
}

pub use plot::compare as plot_compare;
pub use plot::single as plot_single;

/// Trait for better display of [`Durations`][Duration]
pub trait DurationUtils {
    /// Returns a string with the duration in the format `HH:MM:SS`
    /// or `MM:SS` (if the duration is less than an hour)
    fn display(&self) -> String;
}
impl DurationUtils for Duration {
    fn display(&self) -> String {
        let hours = self.num_hours();
        let seconds = self.num_seconds() % 60;
        if hours > 0 {
            let minutes = self.num_minutes() % hours;
            format!("{hours:02}:{minutes:02}:{seconds:02}")
        } else {
            let minutes = self.num_minutes();
            format!("{minutes:02}:{seconds:02}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests the `::new` and `::from_str` constructors of Artist, Album and Song
    #[test]
    fn test_constructors() {
        assert_eq!(Artist::new(String::from("Sabaton")), Artist::new("Sabaton"));
        assert_eq!(
            Artist::new("Sabaton"),
            Artist {
                name: "Sabaton".to_string()
            }
        );

        assert_eq!(
            Album::new(String::from("Coat of Arms"), String::from("Sabaton")),
            Album::new("Coat of Arms", "Sabaton")
        );
        assert_eq!(
            Album::new("Coat of Arms", "Sabaton"),
            Album {
                name: "Coat of Arms".to_string(),
                artist: Artist::new("Sabaton")
            }
        );

        assert_eq!(
            Song::new(
                String::from("The Final Solution"),
                String::from("Coat of Arms"),
                String::from("Sabaton")
            ),
            Song::new("The Final Solution", "Coat of Arms", "Sabaton")
        );
        assert_eq!(
            Song::new("The Final Solution", "Coat of Arms", "Sabaton"),
            Song {
                name: "The Final Solution".to_string(),
                album: Album::new("Coat of Arms", "Sabaton")
            }
        );
    }

    /// Tests [`PartialOrd`] and [`Ord`] for [`Artist`]
    #[test]
    fn ord_artist() {
        assert!(Artist::new("Sabaton") > Artist::new("Sabatoa"));
        assert!(
            Artist::new("Sabaton").partial_cmp(&Artist::new("Sabatoa")) == Some(Ordering::Greater)
        );

        assert!(Artist::new("Sabaton") == Artist::new("Sabaton"));
        assert!(
            Artist::new("Sabaton").partial_cmp(&Artist::new("Sabaton")) == Some(Ordering::Equal)
        );

        assert!(Artist::new("Sabaton") < Artist::new("Sabatoz"));
        assert!(
            Artist::new("Sabaton").partial_cmp(&Artist::new("Sabatoz")) == Some(Ordering::Less)
        );
    }

    /// Tests [`PartialOrd`] and [`Ord`] for [`Album`]
    #[test]
    fn ord_album() {
        assert!(Album::new("Coat of Arms", "Sabaton") > Album::new("Carolus Rex", "Sabaton"));
        assert!(
            Album::new("Coat of Arms", "Sabaton")
                .partial_cmp(&Album::new("Carolus Rex", "Sabaton"))
                == Some(Ordering::Greater)
        );

        assert!(Album::new("AAAA", "ZZZZZ") > Album::new("Carolus Rex", "Sabaton"));
        assert!(
            Album::new("AAAA", "ZZZZZ").partial_cmp(&Album::new("Carolus Rex", "Sabaton"))
                == Some(Ordering::Greater)
        );

        assert!(Album::new("Carolus Rex", "Sabaton") == Album::new("Carolus Rex", "Sabaton"));
        assert!(
            Album::new("Carolus Rex", "Sabaton").partial_cmp(&Album::new("Carolus Rex", "Sabaton"))
                == Some(Ordering::Equal)
        );

        assert!(Album::new("ZZZZZZZ", "Alestorm") < Album::new("AAAAAA", "Sabaton"));
        assert!(
            Album::new("ZZZZZZZ", "Alestorm").partial_cmp(&Album::new("AAAAAA", "Sabaton"))
                == Some(Ordering::Less)
        );
    }

    /// Tests [`PartialOrd`] and [`Ord`] for [`Song`]
    #[test]
    fn ord_song() {
        assert!(
            Song::new("Swedish Pagans", "Carolus Rex", "Sabaton")
                > Song::new("Coat of Arms", "Coat of Arms", "Sabaton")
        );
        assert!(
            Song::new("Swedish Pagans", "Carolus Rex", "Sabaton").partial_cmp(&Song::new(
                "Coat of Arms",
                "Coat of Arms",
                "Sabaton"
            )) == Some(Ordering::Greater)
        );

        assert!(
            Song::new(
                "Forbidden Equations Deep Within The Epimethean Wasteland",
                "Hypercube Necrodimensions",
                "Wizardthrone"
            ) > Song::new(
                "Forbidden Equations Deep Within The Epimethean Wasteland",
                "Forbidden Equations Deep Within The Epimethean Wasteland",
                "Wizardthrone"
            )
        );
        assert!(
            Song::new(
                "Forbidden Equations Deep Within The Epimethean Wasteland",
                "Hypercube Necrodimensions",
                "Wizardthrone"
            )
            .partial_cmp(&Song::new(
                "Forbidden Equations Deep Within The Epimethean Wasteland",
                "Forbidden Equations Deep Within The Epimethean Wasteland",
                "Wizardthrone"
            )) == Some(Ordering::Greater)
        );

        assert!(
            Song::new("Swedish Pagans", "Carolus Rex", "Sabaton")
                == Song::new("Swedish Pagans", "Carolus Rex", "Sabaton")
        );
        assert!(
            Song::new("Swedish Pagans", "Carolus Rex", "Sabaton").partial_cmp(&Song::new(
                "Swedish Pagans",
                "Carolus Rex",
                "Sabaton"
            )) == Some(Ordering::Equal)
        );

        assert!(
            Song::new("Hearts on Fire", "Crimson Thunder", "HammerFall")
                < Song::new("The Final Solution", "Coat of Arms", "Sabaton")
        );
        assert!(
            Song::new("Hearts on Fire", "Crimson Thunder", "HammerFall").partial_cmp(&Song::new(
                "The Final Solution",
                "Coat of Arms",
                "Sabaton"
            )) == Some(Ordering::Less)
        );
    }

    #[test]
    fn test_dates() {
        // MAYBE RATHER INTEGRATION TEST THAN UNIT TEST?!
        let paths = vec![format!(
            "{}/stuff/example_endsong/endsong_0.json",
            std::env::current_dir().unwrap().display()
        )];
        let entries = crate::types::SongEntries::new(&paths).unwrap();

        let first = entries
            .iter()
            .min_by(|x, y| x.timestamp.cmp(&y.timestamp))
            .unwrap() // unwrap ok bc there is at least one entry
            .timestamp;
        assert_eq!(first, entries.first_date());

        let last = entries
            .iter()
            .max_by(|x, y| x.timestamp.cmp(&y.timestamp))
            .unwrap() // unwrap ok bc there is at least one entry
            .timestamp;
        assert_eq!(last, entries.last_date());
    }
}
