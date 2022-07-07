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
