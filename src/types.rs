//! Module containg many types used throughout the program
// https://doc.rust-lang.org/stable/book/ch06-01-defining-an-enum.html
use std::fmt::Display;

use chrono::DateTime;

pub enum AspectFull<'a> {
    Artist(&'a Artist),
    Album(&'a Album),
    Song(&'a Song),
}

// you can derive Default in Rust 1.62 https://github.com/rust-lang/rust/pull/94457/
#[derive(Default)]
pub enum Aspect {
    Artists,
    Albums,
    // bc Rust still doesn't have default argument values
    // https://www.reddit.com/r/rust/comments/fi6nov/why_does_rust_not_support_default_arguments/fkfezxv/
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} - {} ({})",
            self.album.artist.name, self.name, self.album.name
        )
    }
}
impl Music for Song {}

#[derive(Clone, Debug)]
pub struct SongEntry {
    pub timestamp: DateTime<chrono::FixedOffset>,
    pub ms_played: u32,
    pub track: String,
    pub album: String,
    pub artist: String,
    pub id: String,
}

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
