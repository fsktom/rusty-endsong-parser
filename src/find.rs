//! Module responsible for finding artists, albums and songs in the dataset

use itertools::Itertools;

use crate::aspect::{Album, Artist, Music, Song};
use crate::entry::SongEntry;

/// Searches the entries for if the given artist exists in the dataset
///
/// Case-insensitive and returns the [`Artist`] with proper capitalization
/// (i.e. the capitalization of the first entry it finds)
///
/// See #2 <https://github.com/fsktom/rusty-endsong-parser/issues/2>
pub fn artist(entries: &[SongEntry], artist_name: &str) -> Option<Artist> {
    let usr_artist = Artist::new(artist_name.to_lowercase());

    entries
        .iter()
        .find(|entry| usr_artist.is_entry_lowercase(entry))
        .map(Artist::from)
}

/// Searches the entries for if the given album exists in the dataset
///
/// Case-insensitive and returns the [`Album`] with proper capitalization
/// (i.e. the capitalization of the first entry it finds)
///
/// See #2 <https://github.com/fsktom/rusty-endsong-parser/issues/2>
pub fn album(entries: &[SongEntry], album_name: &str, artist_name: &str) -> Option<Album> {
    let usr_album = Album::new(album_name.to_lowercase(), artist_name.to_lowercase());

    entries
        .iter()
        .find(|entry| usr_album.is_entry_lowercase(entry))
        .map(Album::from)
}

/// Searches the entries for if the given song (in that specific album)
/// exists in the dataset
///
/// Case-insensitive and returns the [`Song`] with proper capitalization
/// (i.e. the capitalization of the first entry it finds)
///
/// See #2 <https://github.com/fsktom/rusty-endsong-parser/issues/2>
pub fn song_from_album(
    entries: &[SongEntry],
    song_name: &str,
    album_name: &str,
    artist_name: &str,
) -> Option<Song> {
    let usr_song = Song::new(
        song_name.to_lowercase(),
        album_name.to_lowercase(),
        artist_name.to_lowercase(),
    );

    entries
        .iter()
        .find(|entry| usr_song.is_entry_lowercase(entry))
        .map(Song::from)
}

/// Searches the dataset for multiple versions of a song
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
        .unique()
        .map(Song::from)
        .collect_vec();

    if song_versions.is_empty() {
        return None;
    }

    Some(song_versions)
}

/// Returns a [`Vec<Song>`] with all the songs in the given album
///
/// # Panics
///
/// Panics if `album` is not in the dataset
pub fn songs_from_album(entries: &[SongEntry], album: &Album) -> Vec<Song> {
    entries
        .iter()
        .filter(|entry| album.is_entry(entry))
        .unique()
        .map(Song::from)
        .collect_vec()
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
            artist(&entries, "Theocracy").unwrap(),
            Artist::new("Theocracy")
        );
        assert!(entries.find().artist("Powerwolf").is_none());
    }
}
