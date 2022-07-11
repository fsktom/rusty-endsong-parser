//! Module containg many types used throughout the program
// https://doc.rust-lang.org/stable/book/ch06-01-defining-an-enum.html
use std::fmt::Display;

use chrono::DateTime;

use crate::display;
use crate::parse;

/// Algebraic data type similar to [Aspect]
/// but used by functions such as [crate::display::print_aspect]
/// to get more specfic data
///
/// Each variant contains a reference to an instance of the aspect
pub enum AspectFull<'a> {
    /// with ref to [Artist]
    Artist(&'a Artist),
    /// with ref to [Album]
    Album(&'a Album),
    /// with ref to [Song]
    Song(&'a Song),
}

// you can derive Default in Rust 1.62 https://github.com/rust-lang/rust/pull/94457/
/// An enum that is among other things used by functions such as
/// [crate::display::print_top] and its derivatives to know whether
/// to print top songs ([Aspect::Songs]), albums ([Aspect::Albums])
/// or artists ([Aspect::Artists])
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

/// Used for functions in [crate::display] that accept either
/// a [Song], [Album] or [Artist] struct
pub trait Music: Display {}

/// Struct for representing an artist
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct Artist {
    /// Name of the artist
    pub name: String,
}
impl Artist {
    /// Creates an instance of Artist with a [String] parameter
    pub fn new(artist_name: String) -> Artist {
        Artist { name: artist_name }
    }

    /// Creates an instance of Artist with a &[str] parameter
    pub fn from_str(artist_name: &str) -> Artist {
        Artist {
            name: artist_name.to_string(),
        }
    }
}
impl Display for Artist {
    /// Formats the struct in "<artist_name>" format
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
impl Music for Artist {}

/// Struct for representing an album
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct Album {
    /// Name of the album
    pub name: String,
    /// Artist of the album
    pub artist: Artist,
}
impl Album {
    /// Creates an instance of Album with [String] parameters
    pub fn new(album_name: String, artist_name: String) -> Album {
        Album {
            name: album_name,
            artist: Artist::new(artist_name),
        }
    }

    /// Creates an instance of Album with &[str] parameters
    pub fn from_str(album_name: &str, artist_name: &str) -> Album {
        Album {
            name: album_name.to_string(),
            artist: Artist::from_str(artist_name),
        }
    }
}
impl Display for Album {
    /// Formats the struct in "<artist_name> - <album_name>" format
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.artist.name, self.name)
    }
}
impl Music for Album {}

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
    /// Creates an instance of Song with [String] parameters
    pub fn new(song_name: String, album_name: String, artist_name: String) -> Song {
        Song {
            name: song_name,
            album: Album::new(album_name, artist_name),
        }
    }

    /// Creates an instance of Song with &[str] parameters
    pub fn from_str(song_name: &str, album_name: &str, artist_name: &str) -> Song {
        Song {
            name: song_name.to_string(),
            album: Album::from_str(album_name, artist_name),
        }
    }
}
impl Display for Song {
    /// Formats the struct in "<artist_name> - <song_name> (<album_name>)" format
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} - {} ({})",
            self.album.artist.name, self.name, self.album.name
        )
    }
}
impl Music for Song {}

/// A more specific version of [crate::parse::Entry]
/// utilized by many functions here.
/// Only for entries which are songs (there are also podcast entries)
/// Contains the relevant metadata of each entry song entry in endsong.json
#[derive(Clone, Debug)]
pub struct SongEntry {
    /// the time at which the song has been played
    pub timestamp: DateTime<chrono::FixedOffset>,
    /// for how long the song has been played
    pub ms_played: u32,
    /// name of the song
    pub track: String,
    /// name of the album
    pub album: String,
    /// name of the artist
    pub artist: String,
    /// Spotify URI
    pub id: String,
}

/// Struct containing a vector of [SongEntry]
///
/// Fundamental for the use of this program
pub struct SongEntries(Vec<SongEntry>);

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

impl SongEntries {
    /// Creates an instance of SongEntries
    ///
    /// * `paths` - a vector containing paths to each `endsong.json` file
    pub fn new(paths: Vec<String>) -> SongEntries {
        SongEntries(parse::parse(paths))
    }

    /// Prints the top `num` of an `asp`
    ///
    /// * `asp` - [Aspect::Songs] (affected by [display::SUM_ALBUMS]) for top songs, [Aspect::Albums] for top albums and
    /// [Aspect::Artists] for top artists
    /// * `num` - number of displayed top aspects.
    /// Will automatically change to total number of that aspect if `num` is higher than that
    ///
    /// Wrapper for [display::print_top()]
    pub fn print_top(&self, asp: Aspect, num: usize) {
        display::print_top(self, asp, num)
    }

    /// Prints top songs or albums from an artist
    ///
    /// * `asp` - [Aspect::Songs] for top songs and [Aspect::Albums] for top albums
    /// * `artist` - the [Artist] you want the top songs/albums from
    /// * `num` - number of displayed top aspects.
    /// Will automatically change to total number of that aspect if `num` is higher than that
    ///
    /// Wrapper for [display::print_top_from_artist()]
    pub fn print_top_from_artist(&self, asp: Aspect, artist: &Artist, num: usize) {
        display::print_top_from_artist(self, asp, artist, num)
    }

    /// Prints top songs from an album
    ///
    /// * `album` - the [Album] you want the top songs from
    /// * `num` - number of displayed top songs.
    /// Will automatically change to total number of songs from that album if `num` is higher than that
    ///
    /// Wrapper for [display::print_top_from_album()]
    pub fn print_top_from_album(&self, album: &Album, num: usize) {
        display::print_top_from_album(self, album, num)
    }

    /// Prints a specfic aspect
    ///
    /// * `asp` - the aspect you want informationa about containing the
    /// relevant struct
    ///
    /// Wrapper for [display::print_aspect()]
    pub fn print_aspect(&self, asp: AspectFull) {
        display::print_aspect(self, asp)
    }

    /// Adds search capability
    ///
    /// Use with methods from [Find]: [`.artist()`](Find::artist()), [`.album()`](Find::album()),
    /// [`.song_from_album()`](Find::song_from_album()) and [`.song()`](Find::song())
    pub fn find(&self) -> Find {
        Find(self)
    }
}

/// Used by [SongEntries] as a wrapper for
/// [display::find_artist()], [display::find_album()],
/// [display::find_song_from_album()] and [display::find_song()]
///
/// # Examples
///
/// ```
/// let entries = SongEntries::new(paths);
/// dbg!(entries.find().artist("Sabaton".to_string()));
/// ```
pub struct Find<'a>(&'a SongEntries);

// https://users.rust-lang.org/t/how-can-i-return-reference-of-the-struct-field/36325/2
// so that when you use &self it refers to &self.0 (SongEntries,
// which itself refers to Vec<SongEntry> xDD
impl<'a> std::ops::Deref for Find<'a> {
    type Target = SongEntries;
    fn deref(&self) -> &SongEntries {
        self.0
    }
}

impl<'a> Find<'a> {
    /// Searches the entries for if the given artist exists in the dataset
    ///
    /// Wrapper for [display::find_artist()]
    pub fn artist(&self, artist_name: String) -> Option<Artist> {
        display::find_artist(self, artist_name)
    }

    /// Searches the entries for if the given album exists in the dataset
    ///
    /// Wrapper for [display::find_album()]
    pub fn album(&self, album_name: String, artist_name: String) -> Option<Album> {
        display::find_album(self, album_name, artist_name)
    }

    /// Searches the entries for if the given song (in that specific album)
    /// exists in the dataset
    ///
    /// Wrapper for [display::find_song_from_album()]
    pub fn song_from_album(
        &self,
        song_name: String,
        album_name: String,
        artist_name: String,
    ) -> Option<Song> {
        display::find_song_from_album(self, song_name, album_name, artist_name)
    }

    /// Searches the dataset for multiple versions of a song
    ///
    /// Returns a [Vec<Song>] containing an instance
    /// of [Song] for every album it's been found in
    ///
    /// Wrapper for [display::find_song()]
    pub fn song(&self, song_name: String, artist_name: String) -> Option<Vec<Song>> {
        display::find_song(self, song_name, artist_name)
    }
}

/// A more specific version of [crate::parse::Entry]
/// for podcast entries.
#[derive(Clone, Debug)]
pub struct PodcastEntry {}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests the `::new` and `::from_str` constructors of Artist, Album and Song
    #[test]
    fn test_constructors() {
        assert_eq!(
            Artist::new(String::from("Sabaton")),
            Artist::from_str("Sabaton")
        );
        assert_eq!(
            Artist::from_str("Sabaton"),
            Artist {
                name: "Sabaton".to_string()
            }
        );

        assert_eq!(
            Album::new(String::from("Coat of Arms"), String::from("Sabaton")),
            Album::from_str("Coat of Arms", "Sabaton")
        );
        assert_eq!(
            Album::from_str("Coat of Arms", "Sabaton"),
            Album {
                name: "Coat of Arms".to_string(),
                artist: Artist::from_str("Sabaton")
            }
        );

        assert_eq!(
            Song::new(
                String::from("The Final Solution"),
                String::from("Coat of Arms"),
                String::from("Sabaton")
            ),
            Song::from_str("The Final Solution", "Coat of Arms", "Sabaton")
        );
        assert_eq!(
            Song::from_str("The Final Solution", "Coat of Arms", "Sabaton"),
            Song {
                name: "The Final Solution".to_string(),
                album: Album::from_str("Coat of Arms", "Sabaton")
            }
        );
    }
}
