//! Module containing representations of songs, albums, artists and their traits
//!
//! If you want to make a function that accepts either a [`Song`], [`Album`] or [`Artist`] struct,
//! use the [`Music`] trait. It's got methods for comparing with a [`SongEntry`].
//! ```
//! use endsong::prelude::*;
//! fn foo<Asp: Music>(asp: &Asp, entry: &SongEntry) -> bool {
//!     asp.is_entry(entry)
//! }
//!
//! ```
//!
//! If you want to make a function that extracts the artist from either of them, use the [`AsRef<Artist>`] trait.
//! ```
//! use endsong::prelude::*;
//! fn foo<Asp: AsRef<Artist>>(has_art: &Asp) {
//!     let artist: &Artist = has_art.as_ref();
//!     // do stuff with artist
//! }
//! ```
//!
//! If you want to make a function that accepts either an [`Album`] or [`Song`],
//! use the [`AsRef<Artist>`] with [`Music`] trait (which contains [`Display`] impls etc).
//! To then get the artist, use `as_ref()`.
//! ```
//! use endsong::prelude::*;
//! fn foo<'a, Asp: AsRef<Artist> + Music>(asp: &'a Asp, entry: &SongEntry) -> &'a Artist {
//!     println!("{asp}");
//!     asp.as_ref()
//! }
//!
//! ```
//!
//! If you want to make a function that extracts the album from [`Album`] or [`Song`], use the [`AsRef<Album>`] trait.
//! ```
//! use endsong::prelude::*;
//! fn foo<Asp: AsRef<Album>>(has_alb: &Asp) {
//!     let album: &Album = has_alb.as_ref();
//!     // do stuff with album
//! }
//! ```
//!
//! You can also freely create insances of e.g. [`Artist`] and [`Album`] from [`Song`] using its [`From`] impls.
//! See the specific struct [`From`] and [`AsRef`] impls for more info.
//!
//! Cloning each aspect or using [`From`] another aspect is O(1) because they use [`Rc`] internally.

use std::cmp::Ordering;
use std::fmt::Display;
use std::rc::Rc;

use crate::entry::SongEntry;

/// Used for functions that accept either
/// a [`Song`], [`Album`] or [`Artist`] struct
pub trait Music: Display + Clone + Eq + Ord {
    /// Checks if a [`SongEntry`] is a [`Music`]
    fn is_entry(&self, entry: &SongEntry) -> bool;

    /// Checks if a [`SongEntry`] is a [`Music`] but case insensitive
    ///
    /// Performs `.to_lowercase()` on both `entry` and on [`self`].
    fn is_entry_lowercase(&self, entry: &SongEntry) -> bool;
}

/// Trait used to accept only [`Artist`] and [`Album`]
pub trait HasSongs: Music {}

/// Struct for representing an artist
#[derive(PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct Artist {
    /// Name of the artist
    pub name: Rc<str>,
}
impl Artist {
    /// Creates an instance of Artist
    pub fn new<S: Into<Rc<str>>>(artist_name: S) -> Artist {
        Artist {
            name: artist_name.into(),
        }
    }
}
impl Clone for Artist {
    /// Clones the artist
    /// with an [`Rc`], so cost of clone is O(1)
    fn clone(&self) -> Self {
        Artist {
            name: Rc::clone(&self.name),
        }
    }
}
impl Display for Artist {
    /// Formats the struct in "<`artist_name`>" format
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
impl From<&Artist> for Artist {
    /// Clones the artist
    /// with an [`Rc`], so cost of clone is O(1)
    fn from(artist: &Artist) -> Self {
        artist.clone()
    }
}
impl From<&Album> for Artist {
    /// Clones the artist of `alb`
    /// with an [`Rc`], so cost of clone is O(1)
    fn from(alb: &Album) -> Self {
        alb.artist.clone()
    }
}
impl From<&Song> for Artist {
    /// Clones the artist of `son`
    /// with an [`Rc`], so cost of clone is O(1)
    fn from(son: &Song) -> Self {
        son.album.artist.clone()
    }
}
impl From<&SongEntry> for Artist {
    /// Creates an instance of Artist from a ref to [`SongEntry`]
    ///
    /// Clones the artist name from `entry` with an [`Rc`],
    /// so cost of clone is O(1)
    fn from(entry: &SongEntry) -> Self {
        Artist {
            name: Rc::clone(&entry.artist),
        }
    }
}
impl AsRef<Artist> for Artist {
    fn as_ref(&self) -> &Artist {
        self
    }
}
impl Music for Artist {
    fn is_entry(&self, entry: &SongEntry) -> bool {
        entry.artist == self.name
    }
    fn is_entry_lowercase(&self, entry: &SongEntry) -> bool {
        entry.artist.to_lowercase() == self.name.to_lowercase()
    }
}
impl HasSongs for Artist {}

/// Struct for representing an album
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Album {
    /// Name of the album
    pub name: Rc<str>,
    /// Artist of the album
    pub artist: Artist,
}
impl Album {
    /// Creates an instance of Album
    pub fn new<S: Into<Rc<str>>>(album_name: S, artist_name: S) -> Album {
        Album {
            name: album_name.into(),
            artist: Artist::new(artist_name),
        }
    }
}
impl Clone for Album {
    /// Clones the album
    /// with an [`Rc`], so cost of clone is O(1)
    fn clone(&self) -> Self {
        Album {
            name: Rc::clone(&self.name),
            artist: self.artist.clone(),
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
impl From<&Album> for Album {
    /// Clones the album with an [`Rc`],
    /// so cost of clone is O(1)
    fn from(album: &Album) -> Self {
        album.clone()
    }
}
impl From<&Song> for Album {
    /// Clones the album of `son` with an [`Rc`],
    /// so cost of clone is O(1)
    fn from(son: &Song) -> Self {
        son.album.clone()
    }
}
impl From<&SongEntry> for Album {
    /// Creates an instance of Album from a ref to [`SongEntry`]
    ///
    /// Clones the album and artist name from `entry` with an [`Rc`],
    /// so cost of clone is O(1)
    fn from(entry: &SongEntry) -> Self {
        Album {
            name: Rc::clone(&entry.album),
            artist: Artist::from(entry),
        }
    }
}
impl AsRef<Album> for Album {
    fn as_ref(&self) -> &Album {
        self
    }
}
impl AsRef<Artist> for Album {
    fn as_ref(&self) -> &Artist {
        &self.artist
    }
}
impl Music for Album {
    fn is_entry(&self, entry: &SongEntry) -> bool {
        entry.artist == self.artist.name && entry.album == self.name
    }
    fn is_entry_lowercase(&self, entry: &SongEntry) -> bool {
        entry.artist.to_lowercase() == self.artist.name.to_lowercase()
            && entry.album.to_lowercase() == self.name.to_lowercase()
    }
}
impl HasSongs for Album {}

/// Struct for representing a song
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Song {
    /// Name of the song
    pub name: Rc<str>,
    /// The album this song is from
    pub album: Album,
    // pub id: Rc<str>,
}
impl Song {
    /// Creates an instance of Song
    pub fn new<S: Into<Rc<str>>>(song_name: S, album_name: S, artist_name: S) -> Song {
        Song {
            name: song_name.into(),
            album: Album::new(album_name, artist_name),
        }
    }
}
impl Clone for Song {
    /// Clones the song
    /// with an [`Rc`], so cost of clone is O(1)
    fn clone(&self) -> Self {
        Song {
            name: Rc::clone(&self.name),
            album: self.album.clone(),
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
impl From<&Song> for Song {
    /// Clones the song
    fn from(song: &Song) -> Self {
        song.clone()
    }
}
impl From<&SongEntry> for Song {
    /// Creates an instance of Song from a ref to [`SongEntry`]
    ///
    /// Clones the song, album and artist name from `entry` with an [`Rc`],
    /// so cost of clone is O(1)
    fn from(entry: &SongEntry) -> Self {
        Song {
            name: Rc::clone(&entry.track),
            album: Album::from(entry),
        }
    }
}
impl AsRef<Song> for Song {
    fn as_ref(&self) -> &Song {
        self
    }
}
impl AsRef<Artist> for Song {
    fn as_ref(&self) -> &Artist {
        &self.album.artist
    }
}
impl AsRef<Album> for Song {
    fn as_ref(&self) -> &Album {
        &self.album
    }
}
impl Music for Song {
    fn is_entry(&self, entry: &SongEntry) -> bool {
        entry.artist == self.album.artist.name
            && entry.album == self.album.name
            && entry.track == self.name
    }
    fn is_entry_lowercase(&self, entry: &SongEntry) -> bool {
        entry.artist.to_lowercase() == self.album.artist.name.to_lowercase()
            && entry.album.to_lowercase() == self.album.name.to_lowercase()
            && entry.track.to_lowercase() == self.name.to_lowercase()
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
                name: Rc::from("Sabaton")
            }
        );

        assert_eq!(
            Album::new(String::from("Coat of Arms"), String::from("Sabaton")),
            Album::new("Coat of Arms", "Sabaton")
        );
        assert_eq!(
            Album::new("Coat of Arms", "Sabaton"),
            Album {
                name: Rc::from("Coat of Arms"),
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
                name: Rc::from("The Final Solution"),
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
        let entries = crate::entry::SongEntries::new(&paths).unwrap();

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
