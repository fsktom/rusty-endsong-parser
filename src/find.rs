//! Module responsible for finding artists, albums and songs in the dataset
//!
//! Use functions here instead of manually creating aspects to make sure
//! they actually exist in the dataset!
//!
//! ```
//! use endsong::prelude::*;
//!
//! // create SongEntries from a single file
//! let paths = vec![format!(
//!     "{}/stuff/example_endsong/endsong_0.json",
//!     std::env::current_dir().unwrap().display()
//! )];
//! let entries = SongEntries::new(&paths).unwrap();
//!
//! // example artist
//! let artist: Artist = entries.find().artist("sabaTON").unwrap().remove(0);
//! assert_eq!(artist, Artist::new("Sabaton"));
//! ```

use itertools::Itertools;

use crate::aspect::{Album, Artist, Music, Song};
use crate::entry::SongEntry;

/// Searches the entries for possible artists
///
/// Case-insensitive and returns the [`Artist`] with proper capitalization.
/// The vector contains multiple [`Artist`]s if they're called the same,
/// but their names are capitalized diffferently
///
/// Vector is guaranteed to be non-empty if [`Some`]
///
/// See #2 <https://github.com/fsktom/rusty-endsong-parser/issues/2>
pub fn artist(entries: &[SongEntry], artist_name: &str) -> Option<Vec<Artist>> {
    let usr_artist = Artist::new(artist_name);

    let artists = entries
        .iter()
        .filter(|entry| usr_artist.is_entry_lowercase(entry))
        .map(Artist::from)
        .unique()
        .collect_vec();

    if artists.is_empty() {
        return None;
    }

    Some(artists)
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
pub fn album(entries: &[SongEntry], album_name: &str, artist_name: &str) -> Option<Vec<Album>> {
    let usr_album = Album::new(album_name, artist_name);

    let albums = entries
        .iter()
        .filter(|entry| usr_album.is_entry_lowercase(entry))
        .map(Album::from)
        .unique()
        .collect_vec();

    if albums.is_empty() {
        return None;
    }

    Some(albums)
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
pub fn song_from_album(
    entries: &[SongEntry],
    song_name: &str,
    album_name: &str,
    artist_name: &str,
) -> Option<Vec<Song>> {
    let usr_song = Song::new(song_name, album_name, artist_name);

    let songs = entries
        .iter()
        .filter(|entry| usr_song.is_entry_lowercase(entry))
        .map(Song::from)
        .unique()
        .collect_vec();

    if songs.is_empty() {
        return None;
    }

    Some(songs)
}

/// Searches the dataset for multiple versions of a song
/// (i.e. if a song with the same name is in multiple albums)
///
/// Case-insensitive and returns a [`Vec<Song>`] containing an instance
/// of [`Song`] for every album it's been found in with proper capitalization
///
/// See #2 <https://github.com/fsktom/rusty-endsong-parser/issues/2>
pub fn song(entries: &[SongEntry], song_name: &str, artist_name: &str) -> Option<Vec<Song>> {
    let (song_name, artist_name) = (song_name.to_lowercase(), artist_name.to_lowercase());

    let song_versions = entries
        .iter()
        .filter(|entry| {
            entry.track.to_lowercase() == song_name && entry.artist.to_lowercase() == artist_name
        })
        .map(Song::from)
        .unique()
        .collect_vec();

    if song_versions.is_empty() {
        return None;
    }

    Some(song_versions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_aspect() {
        // MAYBE RATHER INTEGRATION TEST THAN UNIT TEST?!
        let paths = vec![format!(
            "{}/stuff/example_endsong/endsong_0.json",
            std::env::current_dir().unwrap().display()
        )];
        let entries = crate::entry::SongEntries::new(&paths).unwrap();

        assert_eq!(
            artist(&entries, "Theocracy").unwrap()[0],
            Artist::new("Theocracy")
        );
        assert!(entries.find().artist("Powerwolf").is_none());
    }
}
