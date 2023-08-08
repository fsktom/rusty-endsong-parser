//! Module containing representation of a single song stream in endsong.json [`SongEntry`]
//! and [`SongEntries`] which is a collection of [`SongEntry`]s

use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::rc::Rc;

use chrono::{DateTime, Duration, Local};
use itertools::Itertools;

use crate::aspect;
use crate::find;
use crate::gather;
use crate::parse;

use aspect::{Album, Artist, HasSongs, Music, Song};

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
    pub time_played: Duration,
    /// name of the song
    pub track: Rc<str>,
    /// name of the album
    pub album: Rc<str>,
    /// name of the artist
    pub artist: Rc<str>,
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
    /// # Arguments
    ///
    /// * `paths` - a slice of [`Paths`][`Path`] to each `endsong.json` file.
    /// Those can be [`Strings`][String], [`strs`][str], [`PathBufs`][std::path::PathBuf] or whatever implements [`AsRef<Path>`]
    ///
    /// # Errors
    ///
    /// Will return an error if any of the files can't be opened or read
    pub fn new<P: AsRef<Path>>(paths: &[P]) -> Result<SongEntries, Box<dyn Error>> {
        Ok(SongEntries(parse::parse(paths)?))
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
    #[must_use]
    pub fn max_listening_time(
        &self,
        time_span: Duration,
    ) -> (Duration, DateTime<Local>, DateTime<Local>) {
        let first = self.first_date();
        let last = self.last_date();

        let actual_time_span = match time_span {
            // maximum duration is whole dataset?
            x if x >= last - first => {
                return (gather::listening_time(self), first, last);
            }
            // minimum duration is 1 day
            x if x < Duration::days(1) => Duration::days(1),
            // duration is within bounds
            _ => time_span,
        };

        let mut highest = Duration::seconds(0);
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
            start += Duration::days(1);
            end += Duration::days(1);
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
    #[must_use]
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
    #[must_use]
    pub fn gather_plays_of_many<Asp: Music>(&self, aspects: &[Asp]) -> usize {
        gather::plays_of_many(self, aspects)
    }

    /// Adds search capability
    ///
    /// Use with methods from [`Find`]: [`.artist()`][Find::artist()], [`.album()`][Find::album()],
    /// [`.song_from_album()`][Find::song_from_album()] and [`.song()`][Find::song()]
    #[must_use]
    pub fn find(&self) -> Find {
        Find(self)
    }

    /// Returns a [`HashMap`] with the [`Songs`][Song] as keys and
    /// their [`Durations`][Duration] as values
    ///
    /// # Panics
    ///
    /// Panics if the dataset is empty? (but that should never happen)
    #[must_use]
    pub fn song_durations(&self) -> HashMap<Song, Duration> {
        // 10k is just a guess for amount of unique songs
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
    /// or below a certain absolute [`Duration`]
    ///
    /// # Arguments
    ///
    /// `percent_threshold` - a value between 0 and 100 (%); a good default is `30`
    /// `absolute_threshold` - all songs below this [`Duration`]
    /// will be filtered out; a good default is `Duration::seconds(10)`
    ///
    /// # Panics
    ///
    /// Will panic if `threshhold` is below 0 or above 100
    pub fn filter(&mut self, percent_threshold: i32, absolute_threshold: Duration) {
        assert!(
            (0..=100).contains(&percent_threshold),
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

            entry.time_played >= (*dur * percent_threshold) / 100
                && entry.time_played >= absolute_threshold
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
    #[must_use]
    pub fn artist(&self, artist_name: &str) -> Option<Artist> {
        find::artist(self.0, artist_name)
    }

    /// Searches the entries for if the given album exists in the dataset
    ///
    /// Case-insensitive and returns the [`Album`] with proper capitalization
    /// (i.e. the capitalization of the first entry it finds)
    ///
    /// See #2 <https://github.com/fsktom/rusty-endsong-parser/issues/2>
    #[must_use]
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
    #[must_use]
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
    #[must_use]
    pub fn song(&self, song_name: &str, artist_name: &str) -> Option<Vec<Song>> {
        find::song(self.0, song_name, artist_name)
    }

    /// Returns a [`Vec<Song>`] with all the songs in the given album
    ///
    /// # Panics
    ///
    /// Panics if `album` is not in the dataset
    #[must_use]
    pub fn songs_from_album(&self, album: &Album) -> Vec<Song> {
        find::songs_from_album(self.0, album)
    }
}
