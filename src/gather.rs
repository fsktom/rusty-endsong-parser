//! Module responsible for gathering artists, albums and songs with their playcounts
//!
//! These functions take in a slice of [`SongEntry`]s. If you want get data
//! between certain dates use [`SongEntries::between`][crate::entry::SongEntries::between]
//! to get a slice of entries between two dates and then pass that slice to these functions.
//!
//! Using [`&SongEntries`][crate::entry::SongEntries] is also possible for data for the whole dataset
//! since it implements [`Deref`][std::ops::Deref] to the [`Vec<SongEntry>`] it contains.
//!
//! Use [`get_sorted_list`][crate::get_sorted_list] and
//! [`get_sorted_ref_list`][crate::get_sorted_ref_list] to transform the [`HashMap`]s
//! from the functions here into [`Vec`]s sorted by playcount
//!
//! # Examples
//! ```rust
//! use endsong::prelude::*;
//! use itertools::Itertools;
//!
//! // create SongEntries from a single file
//! let paths = vec![format!(
//!     "{}/stuff/example_endsong/endsong_0.json",
//!     std::env::current_dir().unwrap().display()
//! )];
//! let entries = SongEntries::new(&paths).unwrap();
//!
//! // example artist
//! let artist: Artist = entries.find().artist("Sabaton").unwrap().remove(0);
//!
//! // get all albums from the artist with their plays
//! let _ = gather::albums_from_artist(&entries, &artist);
//!
//! // get albums from the artist in a given time period
//! let start_date = parse_date("2020-11-14").unwrap();
//! let end_date = parse_date("now").unwrap();
//! let _ = gather::albums_from_artist(entries.between(&start_date, &end_date), &artist);
//!
//! // to get a list of albums from the artist sorted
//! // primarily by their playcount descending
//! // and then alphabetically
//! let albums_map = gather::albums_from_artist(&entries, &artist);
//! let albums: Vec<&Album> = get_sorted_ref_list(&albums_map);
//! let albums_owned: Vec<Album> = get_sorted_list(gather::albums_from_artist(&entries, &artist));
//! ```

use std::collections::HashMap;
use std::rc::Rc;

use chrono::TimeDelta;
use itertools::Itertools;

use crate::aspect::{Album, Artist, HasSongs, Music, Song};
use crate::entry::SongEntry;

/// Returns a map with all [`Songs`][Song] and their playcount while taking
/// the album the song is in into account
///
/// See [`songs_summed_across_albums`] for a version which ignores the album
#[must_use]
pub fn songs(entries: &[SongEntry]) -> HashMap<Song, usize> {
    entries.iter().map(Song::from).counts()
}

/// Like [`songs`] but summarizes the number of plays of a song if the song
/// and artist name are the same -> ignores the album
///
/// It matters because oftentimes the same song will be in many albums (or singles).
/// But it's still case-sensitive!
///
/// # Panics
///
/// Uses .`unwrap()` but it should never panic
#[must_use]
pub fn songs_summed_across_albums(entries: &[SongEntry]) -> HashMap<Song, usize> {
    let songs = entries.iter().map(Song::from).counts();

    // to know which album the song had highest amount of plays from
    // that album will be then displayed in () after the song name
    // but the number of plays that will be displayed will be a sum of
    // the plays from all albums
    // key: (song name, artist)
    // value: HashMap of albums with number of plays of the song in that album
    let mut songs_albums: HashMap<(Rc<str>, Artist), HashMap<Album, usize>> =
        HashMap::with_capacity(songs.len());
    for (song, plays_song) in songs {
        let song_just_artist = (song.name, song.album.artist.clone());

        songs_albums
            .entry(song_just_artist)
            .or_default()
            .insert(song.album, plays_song);
    }

    // required because only one version (i.e. album) of the song should be saved
    let mut songs: HashMap<Song, usize> = HashMap::with_capacity(songs_albums.len());

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
pub fn listening_time(entries: &[SongEntry]) -> TimeDelta {
    entries.iter().map(|entry| entry.time_played).sum()
}
