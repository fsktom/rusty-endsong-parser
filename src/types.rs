//! Module containg many types used throughout the program
// https://doc.rust-lang.org/stable/book/ch06-01-defining-an-enum.html
use std::cmp::Ordering;
use std::error::Error;
use std::fmt::Display;

use chrono::{DateTime, Duration};
use chrono_tz::Tz;
pub use plotly::Trace;

use crate::display;
use crate::parse;
use crate::plot;

/// Algebraic data type similar to [`Aspect`]
/// but used by functions such as [`display::print_aspect()`]
/// to get more specfic data
///
/// Each variant contains a reference to an instance of the aspect
pub enum AspectFull<'a> {
    /// with ref to [`Artist`]
    Artist(&'a Artist),
    /// with ref to [`Album`]
    Album(&'a Album),
    /// with ref to [`Song`]
    Song(&'a Song),
}

/// An enum that is among other things used by functions such as
/// [`display::print_top()`] and its derivatives to know whether
/// to print top songs ([`Aspect::Songs`]), albums ([`Aspect::Albums`])
/// or artists ([`Aspect::Artists`])
#[derive(Default)]
pub enum Aspect {
    /// to print top artists
    Artists,
    /// to print top albums
    Albums,
    // bc Rust still doesn't have default argument values
    // https://www.reddit.com/r/rust/comments/fi6nov/why_does_rust_not_support_default_arguments/fkfezxv/
    /// to print top songs
    #[default]
    Songs,
}
impl Display for Aspect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Aspect::Artists => write!(f, "artists"),
            Aspect::Albums => write!(f, "albums"),
            Aspect::Songs => write!(f, "songs"),
        }
    }
}

/// For choosing mode of a function, similar to [`Aspect`] but
/// without [`Aspect::Artists`]
///
/// Used in [`display::print_top_from_artist()`]
pub enum Mode {
    /// to print albums from artist
    Albums,
    /// to print songs from artists
    Songs,
}

/// Used for functions that accept either
/// a [`Song`], [`Album`] or [`Artist`] struct
pub trait Music: Display + Clone + Eq + Ord {
    /// Checks if a [`SongEntry`] is a [`Music`]
    fn is_entry(&self, entry: &SongEntry) -> bool;
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
impl Music for Artist {
    fn is_entry(&self, entry: &SongEntry) -> bool {
        entry.artist.eq(&self.name)
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
impl Music for Album {
    fn is_entry(&self, entry: &SongEntry) -> bool {
        entry.artist.eq(&self.artist.name) && entry.album.eq(&self.name)
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
impl Music for Song {
    fn is_entry(&self, entry: &SongEntry) -> bool {
        entry.artist.eq(&self.album.artist.name)
            && entry.album.eq(&self.album.name)
            && entry.track.eq(&self.name)
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

/// Struct containing a vector of [`SongEntry`]
///
/// Fundamental for the use of this program
pub struct SongEntries(Vec<SongEntry>);
impl SongEntries {
    /// Creates an instance of [`SongEntries`]
    ///
    /// Returns an [`Error`] if it encounters problems while parsing
    ///
    /// * `paths` - a slice of paths to each `endsong.json` file
    pub fn new<P: AsRef<std::path::Path>>(paths: &[P]) -> Result<SongEntries, Box<dyn Error>> {
        Ok(SongEntries(parse::parse(paths)?))
    }

    /// Returns the date of the first (time-wise) occurrence of any [`SongEntry`]
    pub fn first_date(&self) -> DateTime<Tz> {
        self.iter()
            .min_by(|x, y| x.timestamp.cmp(&y.timestamp))
            .unwrap() // unwrap ok bc there is at least one entry
            .timestamp
    }

    /// Returns the date of the last (time-wise) occurrence of any [`SongEntry`]
    pub fn last_date(&self) -> DateTime<Tz> {
        self.iter()
            .max_by(|x, y| x.timestamp.cmp(&y.timestamp))
            .unwrap() // unwrap ok bc there is at least one entry
            .timestamp
    }

    /// Prints the top `num` of an `asp`
    ///
    /// * `asp` - [`Aspect::Songs`] for top songs, [`Aspect::Albums`]
    ///  for top albums and [`Aspect::Artists`] for top artists
    /// * `num` - number of displayed top aspects.
    /// Will automatically change to total number of that aspect if `num` is higher than that
    /// * `sum_songs_from_different_albums` - only matters if `asp` is [`Aspect::Songs`].
    /// If set to true, it will sum up the plays of
    /// one song across multiple albums it may be in.
    /// The album displayed in the parantheses will be the one it has the
    /// highest amount of listens from.
    pub fn print_top(&self, asp: &Aspect, num: usize, sum_songs_from_different_albums: bool) {
        display::print_top(self, asp, num, sum_songs_from_different_albums);
    }

    /// Prints top songs or albums from an artist
    ///
    /// * `asp` - [`Aspect::Songs`] for top songs and [`Aspect::Albums`] for top albums
    /// * `artist` - the [`Artist`] you want the top songs/albums from
    /// * `num` - number of displayed top aspects.
    /// Will automatically change to total number of that aspect if `num` is higher than that
    ///
    /// Wrapper for [`display::print_top_from_artist()`]
    pub fn print_top_from_artist(&self, mode: &Mode, artist: &Artist, num: usize) {
        display::print_top_from_artist(self, mode, artist, num);
    }

    /// Prints top songs from an album
    ///
    /// * `album` - the [`Album`] you want the top songs from
    /// * `num` - number of displayed top songs.
    /// Will automatically change to total number of songs from that album if `num` is higher than that
    ///
    /// Wrapper for [`display::print_top_from_album()`]
    pub fn print_top_from_album(&self, album: &Album, num: usize) {
        display::print_top_from_album(self, album, num);
    }

    /// Prints a specfic aspect
    ///
    /// * `asp` - the aspect you want informationa about containing the
    /// relevant struct
    ///
    /// Wrapper for [`display::print_aspect()`]
    pub fn print_aspect(&self, asp: &AspectFull) {
        display::print_aspect(self, asp);
    }

    /// Prints a specfic aspect
    ///
    /// Basically [`print_aspect()`][SongEntries::print_aspect()] but with date limitations
    ///
    /// * `asp` - the aspect you want informationa about containing the
    /// relevant struct
    ///
    /// Wrapper for [`display::print_aspect_date()`]
    pub fn print_aspect_date(&self, asp: &AspectFull, start: &DateTime<Tz>, end: &DateTime<Tz>) {
        display::print_aspect_date(self, asp, start, end);
    }

    /// Returns the total time listened
    pub fn total_listening_time(&self) -> Duration {
        // sadly doesn't work bc neither chrono::Duration nor std::time::Duration implement iter::sum :))))
        // self.iter().map(|entry| entry.time_played).sum::<Duration>()
        let mut sum = Duration::milliseconds(0);
        for entry in self.iter() {
            sum = sum + entry.time_played;
        }
        sum
    }

    /// Returns the time listened in a given date period
    pub fn listening_time(&self, start: &DateTime<Tz>, end: &DateTime<Tz>) -> Duration {
        // sadly doesn't work bc neither chrono::Duration nor std::time::Duration implement iter::sum :))))
        // self.iter()
        //     .filter(|entry| crate::display::date::is_between(&entry.timestamp, start, end))
        //     .map(|entry| entry.time_played)
        //     .sum::<Duration>()
        let mut sum = Duration::milliseconds(0);
        for entry in self
            .iter()
            .filter(|entry| entry.timestamp.is_between(start, end))
        {
            // AddAssign is not implemented in time-0.1.45 yet which most recent chrono
            // version 0.4 is using :)))
            // https://github.com/chronotope/chrono/issues/602#issuecomment-1436548077
            // time 0.3 also supports iter::sum :)))
            sum = sum + entry.time_played;
        }
        sum
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
                return (self.total_listening_time(), first, last);
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
            let current = self.listening_time(&start, &end);
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

    /// Returns a [`Vec<String>`] with the names of all [`Artist`]s in the dataset
    pub fn artists(&self) -> Vec<String> {
        let mut vec = self
            .iter()
            .map(|entry| entry.artist.clone())
            .collect::<Vec<String>>();
        vec.sort();
        // sort and dedup to remove duplicates xd
        // https://www.reddit.com/r/rust/comments/38zzbk/best_way_to_remove_duplicates_from_a_veclist/crz84bq/
        vec.dedup();
        vec
    }

    /// Returns a [`Vec<String>`] with the names of the [`Album`]s
    /// corresponding to the `artist`
    pub fn albums(&self, artist: &Artist) -> Vec<String> {
        let mut vec = self
            .iter()
            .filter(|entry| artist.is_entry(entry))
            .map(|entry| entry.album.clone())
            .collect::<Vec<String>>();
        vec.sort();
        // sort and dedup to remove duplicates xd
        // https://www.reddit.com/r/rust/comments/38zzbk/best_way_to_remove_duplicates_from_a_veclist/crz84bq/
        vec.dedup();
        vec
    }

    /// Returns a [`Vec<String>`] with the names of the [`Song`]s
    /// corresponding to the `album`
    pub fn songs<Asp: HasSongs>(&self, aspect: &Asp) -> Vec<String> {
        let mut vec = self
            .iter()
            .filter(|entry| aspect.is_entry(entry))
            .map(|entry| entry.track.clone())
            .collect::<Vec<String>>();
        vec.sort();
        // sort and dedup to remove duplicates xd
        // https://www.reddit.com/r/rust/comments/38zzbk/best_way_to_remove_duplicates_from_a_veclist/crz84bq/
        vec.dedup();
        vec
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

/// Used by [`SongEntries`] as a wrapper for
/// [`display::find_artist()`], [`display::find_album()`],
/// [`display::find_song_from_album()`] and [`display::find_song()`]
///
/// # Examples
///
/// ```
/// let entries = SongEntries::new(paths);
/// dbg!(entries.find().artist("Sabaton"));
/// ```
///
/// # Errors
///
/// Methods can return an [`Err`] with [`NotFoundError`]
pub struct Find<'a>(&'a SongEntries);
impl<'a> Find<'a> {
    /// Searches the entries for if the given artist exists in the dataset
    ///
    /// Wrapper for [`display::find_artist()`]
    ///
    /// # Errors
    ///
    /// This function will return an [`Err`] with [`NotFoundError::Artist`]
    /// if it cannot find an artist with the given name
    pub fn artist(&self, artist_name: &str) -> Result<Artist, NotFoundError> {
        display::find_artist(self, artist_name)
    }

    /// Searches the entries for if the given album exists in the dataset
    ///
    /// Wrapper for [`display::find_album()`]
    ///
    /// # Errors
    ///
    /// This function will return an [`Err`] with [`NotFoundError::Album`]
    /// if it cannot find an album with the given name and artist
    pub fn album(&self, album_name: &str, artist_name: &str) -> Result<Album, NotFoundError> {
        display::find_album(self, album_name, artist_name)
    }

    /// Searches the entries for if the given song (in that specific album)
    /// exists in the dataset
    ///
    /// Wrapper for [`display::find_song_from_album()`]
    ///
    /// # Errors
    ///
    /// This function will return an [`Err`] with [`NotFoundError::Song`]
    /// if it cannot find a song with the given name from the
    /// given album and artist
    pub fn song_from_album(
        &self,
        song_name: &str,
        album_name: &str,
        artist_name: &str,
    ) -> Result<Song, NotFoundError> {
        display::find_song_from_album(self, song_name, album_name, artist_name)
    }

    /// Searches the dataset for multiple versions of a song
    ///
    /// Returns a [`Vec<Song>`] containing an instance
    /// of [`Song`] for every album it's been found in
    ///
    /// Wrapper for [`display::find_song()`]
    pub fn song(&self, song_name: &str, artist_name: &str) -> Result<Vec<Song>, NotFoundError> {
        display::find_song(self, song_name, artist_name)
    }
}
// https://users.rust-lang.org/t/how-can-i-return-reference-of-the-struct-field/36325/2
// so that when you use &self it refers to &self.0 (SongEntries,
// which itself refers to Vec<SongEntry> xDD
impl<'a> std::ops::Deref for Find<'a> {
    type Target = SongEntries;
    fn deref(&self) -> &SongEntries {
        self.0
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
        plot::absolute::aspect(self, aspect)
    }

    /// Returns a trace of the plays relative to all plays
    ///
    /// Wrapper for [`plot::relative::to_all()`]
    pub fn relative<Asp: Music>(&self, aspect: &Asp) -> (Box<dyn Trace>, String) {
        plot::relative::to_all(self, aspect)
    }

    /// Returns a trace of the plays relative to the artist
    ///
    /// Wrapper for [`plot::relative::to_artist()`]
    pub fn relative_to_artist<Asp: HasArtist>(&self, aspect: &Asp) -> (Box<dyn Trace>, String) {
        plot::relative::to_artist(self, aspect)
    }

    /// Returns a trace of the plays relative to the album
    ///
    /// Wrapper for [`plot::relative::to_album()`]
    pub fn relative_to_album(&self, song: &Song) -> (Box<dyn Trace>, String) {
        plot::relative::to_album(self, song)
    }
}
// https://users.rust-lang.org/t/how-can-i-return-reference-of-the-struct-field/36325/2
// so that when you use &self it refers to &self.0 (SongEntries,
// which itself refers to Vec<SongEntry> xDD
impl<'a> std::ops::Deref for Traces<'a> {
    type Target = SongEntries;
    fn deref(&self) -> &SongEntries {
        self.0
    }
}

pub use plot::compare as plot_compare;
pub use plot::single as plot_single;

/// Errors raised by `display::find_*` functions and [`Find`] methods
/// when they don't find an [`Artist`], [`Album`] or [`Song`]
///
/// loosely based on [`std::io::ErrorKind`]
#[derive(Debug)]
pub enum NotFoundError {
    /// Artist with that name was not found
    ///
    /// Error message: "Sorry, I couldn't find any artist with that name!"
    Artist,
    /// Album with that name from that artist was not found
    ///
    /// Error message: "Sorry, I couldn't find any album with that name
    /// from that artist!"
    Album,
    /// Song with that name from that album and artist was not found
    ///
    /// Error message:
    /// "Sorry, I couldn't find any song with
    /// that name from that album and artist!"
    Song,
    /// Song with that name from that artist was not found
    ///
    /// Error message:
    /// "Sorry, I couldn't find any song with
    /// that name from that artist!"
    JustSong,
}
impl Display for NotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotFoundError::Artist => {
                write!(f, "Sorry, I couldn't find any artist with that name!")
            }
            NotFoundError::Album => {
                write!(
                    f,
                    "Sorry, I couldn't find any album with that name from that artist!"
                )
            }
            NotFoundError::Song => {
                write!(
                    f,
                    "Sorry, I couldn't find any song with that name from that album and artist!"
                )
            }
            NotFoundError::JustSong => {
                write!(
                    f,
                    "Sorry, I couldn't find any song with that name from that artist!"
                )
            }
        }
    }
}
impl Error for NotFoundError {}

/// A more specific version of [`parse::Entry`]
/// for podcast entries.
#[derive(Clone, Debug)]
pub struct PodcastEntry {}

/// [`SongEntry`] but for podcasts
pub struct PodEntry {
    /// Spotify URI
    pub id: String,
}

/// ANSI Colors
///
/// See <https://bixense.com/clicolors>
pub enum Color {
    /// Resets the following text with `\x1b[0m`
    Reset,
    /// Makes the following text green with `\x1b[1;32m`
    Green,
    /// Makes the following text light green with `\x1b[0;32m`
    LightGreen,
    /// Makes the following text cyan with `\x1b[1;36m`
    Cyan,
    /// Makes the following text red with `\x1b[1;31m`
    Red,
    /// Makes the following text pink with `\x1b[1;35m`
    Pink,
}
impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::Reset => write!(f, "\x1b[0m"),
            Color::Green => write!(f, "\x1b[1;32m"),
            Color::LightGreen => write!(f, "\x1b[0;32m"),
            Color::Cyan => write!(f, "\x1b[1;36m"),
            Color::Red => write!(f, "\x1b[1;31m"),
            Color::Pink => write!(f, "\x1b[1;35m"),
        }
    }
}

/// Trait for comparing dates (for now)
pub trait IsBetween {
    /// Checks if the given date is between (or equal) to the other two dates
    ///
    /// Can possibly be later used for things other than dates too lol
    fn is_between(&self, start: &Self, end: &Self) -> bool;
}
impl IsBetween for DateTime<Tz> {
    fn is_between(&self, start: &Self, end: &Self) -> bool {
        self >= start && self <= end
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
}
