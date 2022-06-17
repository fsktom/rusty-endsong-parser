// https://doc.rust-lang.org/stable/book/ch06-01-defining-an-enum.html
use std::fmt::Display;

use chrono::DateTime;

pub enum Aspect {
    Artists,
    Albums,
    Songs,
}

// bc Rust still doesn't have default argument values
// https://www.reddit.com/r/rust/comments/fi6nov/why_does_rust_not_support_default_arguments/fkfezxv/
impl Default for Aspect {
    fn default() -> Self {
        Aspect::Songs
    }
}

pub trait Music: Display {}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct Artist {
    pub name: String,
}

impl Display for Artist {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
impl Music for Artist {}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct Album {
    pub name: String,
    pub artist: Artist,
}

impl Display for Album {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.artist.name, self.name)
    }
}
impl Music for Album {}

// to allow for custom HashMap key
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct Song {
    pub name: String,
    pub album: Album,
    pub id: String,
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
