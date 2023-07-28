//! Module responsible for gathering artists, albums and songs with their playcounts
//!
//! These functions take in a slice of [`SongEntry`]s. If you want get data
//! between certain dates use [`SongEntries::between`][crate::entry::SongEntries::between]
//! to get a slice of entries between two dates and then pass that slice to these functions.
//!
//! Using [`&SongEntries`][crate::entry::SongEntries] is also possible for data for the whole dataset
//! since it implements [`Deref`][std::ops::Deref] to the [`Vec<SongEntry>`] it contains.
//!
//! # Examples
//! ```rust
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
//! let artist = Artist::new("Sabaton");
//!
//! // get all albums from the artist
//! let _ = gather::albums_from_artist(&entries, &artist);
//!
//! // get albums from the artist in a given time period
//! let start_date = LOCATION_TZ
//!     .datetime_from_str("2020-12-14T00:00:00Z", "%FT%TZ")
//!     .unwrap();
//! let end_date = LOCATION_TZ
//!     .datetime_from_str("2021-03-01T00:00:00Z", "%FT%TZ")
//!     .unwrap();
//! let _ = gather::albums_from_artist(entries.between(&start_date, &end_date), &artist);
//! ```

use std::collections::HashMap;

use chrono::Duration;
use itertools::Itertools;

use crate::aspect::{Album, Artist, HasSongs, Music, Song};
use crate::entry::SongEntry;

/// Returns a map with all [`Songs`][Song] and their playcount
///
/// `sum_songs_from_different_albums` - with `true` it will summarize the plays
/// of songs if their name and artist is the same;
/// with `false` it will also take into account the album the song is in
///
/// It matters because oftentimes the same song will be in many albums (or singles).
/// But it's still case-sensitive!
///
/// # Panics
///
/// Uses .unwrap() but it should never panic
#[must_use]
pub fn songs(entries: &[SongEntry], sum_songs_from_different_albums: bool) -> HashMap<Song, usize> {
    let songs = entries.iter().map(Song::from).counts();
    if !sum_songs_from_different_albums {
        return songs;
    }

    let length = songs.len();

    // to know which album the song had highest amount of plays from
    // that album will be then displayed in () after the song name
    // but the number of plays that will be displayed will be a sum of
    // the plays from all albums
    // key: (song name, artist)
    // value: HashMap of albums with number of plays of the song in that album
    let mut songs_albums: HashMap<(String, Artist), HashMap<Album, usize>> =
        HashMap::with_capacity(length);
    for (song, plays_song) in songs {
        let song_just_artist = (song.name, song.album.artist.clone());

        songs_albums
            .entry(song_just_artist)
            .or_insert_with(HashMap::new)
            .insert(song.album, plays_song);
    }

    // required because only one version (i.e. album) of the song should be saved
    let mut songs: HashMap<Song, usize> = HashMap::with_capacity(length);

    for ((song_name, _), albs) in songs_albums {
        // number of plays of the song across all albums
        let total = albs.values().sum();
        // album with the highest number of plays
        let highest = albs
            .into_iter()
            // sorts albums alphabetically so that this function is deterministic
            // if different albums have the same highest number of plays
            .sorted_unstable_by(|(a, _), (b, _)| a.cmp(b))
            .max_by(|(_, a), (_, b)| a.cmp(b))
            .map(|(alb, _)| alb)
            // unwrap ok because there's at least one album?
            .unwrap();

        let son: Song = Song {
            name: song_name,
            album: highest,
        };

        songs.insert(son, total);
    }

    songs
}

/// Returns a map with all [`Songs`][Song] corresponding to `asp` with their playcount
#[must_use]
pub fn songs_from<Asp: HasSongs>(entries: &[SongEntry], aspect: &Asp) -> HashMap<Song, usize> {
    entries
        .iter()
        .filter(|entry| aspect.is_entry(entry))
        .map(Song::from)
        .counts()
}

/// Returns a map with all [`Albums`][Album] and their playcount
#[must_use]
pub fn albums(entries: &[SongEntry]) -> HashMap<Album, usize> {
    entries.iter().map(Album::from).counts()
}

/// Returns a map with all [`Albums`][Album] corresponding to `art` with their playcount
///
/// `art` - the artist to find albums of; accepts either [`&Artist`][Artist],
/// [`&Album`][Album] or [`&Song`][Song] (takes the artist field from the latter two)
#[must_use]
pub fn albums_from_artist<HasArtist: AsRef<Artist>>(
    entries: &[SongEntry],
    art: &HasArtist,
) -> HashMap<Album, usize> {
    entries
        .iter()
        .filter(|entry| art.as_ref().is_entry(entry))
        .map(Album::from)
        .counts()
}

/// Returns a map with all [`Artists`][Artist] and their playcount
#[must_use]
pub fn artists(entries: &[SongEntry]) -> HashMap<Artist, usize> {
    entries.iter().map(Artist::from).counts()
}

/// Counts up the plays of an [`Artist`], [`Album`] or [`Song`]
#[must_use]
pub fn plays<Asp: Music>(entries: &[SongEntry], aspect: &Asp) -> usize {
    entries
        .iter()
        .filter(|entry| aspect.is_entry(entry))
        .count()
}

/// Counts up the plays of all [`Artists`][Artist],
/// [`Albums`][Album] or [`Songs`][Song] in a collection
#[must_use]
pub fn plays_of_many<Asp: Music>(entries: &[SongEntry], aspects: &[Asp]) -> usize {
    entries
        .iter()
        .filter(|entry| aspects.iter().any(|aspect| aspect.is_entry(entry)))
        .count()
}

/// Sums all plays
///
/// Just returns the length of the entries slice
#[must_use]
pub fn all_plays(entries: &[SongEntry]) -> usize {
    entries.len()
}

/// Returns the total time listened
#[must_use]
pub fn listening_time(entries: &[SongEntry]) -> Duration {
    // sadly doesn't work bc neither chrono::Duration nor std::time::Duration implement iter::sum :))))
    // self.iter().map(|entry| entry.time_played).sum::<Duration>()
    entries
        .iter()
        .map(|entry| entry.time_played)
        .fold(Duration::zero(), |sum, dur| sum + dur)
}
