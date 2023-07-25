//! Module containing representations of songs, albums, artists and their traits

use std::cmp::Ordering;
use std::fmt::Display;

use crate::entry::SongEntry;

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
