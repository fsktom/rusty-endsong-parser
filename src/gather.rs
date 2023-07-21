//! Module responsible for gathering artists, albums and songs with their playcounts

use std::collections::HashMap;

use chrono::{DateTime, Duration};
use chrono_tz::Tz;
use itertools::Itertools;

use crate::types::{Album, Artist, HasSongs, Music, Song, SongEntry};

/// Returns a map with all [`Songs`][Song] and their playcount
///
/// `sum_songs_from_different_albums` - with `true` it will summarize the plays
/// of songs if their name and artist is the same;
/// with `false` it will also take into account the album the song is in
///
/// It matters because oftentimes the same song will be in many albums (or singles).
/// But it's still case-sensitive!
pub fn songs(entries: &[SongEntry], sum_songs_from_different_albums: bool) -> HashMap<Song, usize> {
    let mut songs = entries.iter().map(Song::from).counts();
    if !sum_songs_from_different_albums {
        return songs;
    }

    // to know which album the song had highest amount of plays from
    // that album will be then displayed in () after the song name
    // but the number of plays that will be displayed will be a sum of
    // the plays from all albums
    // key: (song name, artist)
    // value: HashMap of albums with number of plays of the song in that album
    let mut changed: HashMap<(String, Artist), HashMap<Album, usize>> = HashMap::new();
    for (song, plays_song) in &songs {
        let song_just_artist = (song.name.clone(), song.album.artist.clone());

        changed
            .entry(song_just_artist)
            .or_insert_with(HashMap::new)
            .insert(song.album.clone(), *plays_song);
    }

    // required because only one version (i.e. album) of the song should be saved
    songs.clear();

    for ((song_name, _), albs) in &changed {
        // number of plays of the song across all albums
        let total = albs.iter().map(|(_, plays)| plays).sum();
        // album with the highest number of plays
        let highest = albs
            .iter()
            // sorts albums alphabetically so that this function is deterministic
            // if different albums have the same highest number of plays
            .sorted_unstable_by(|(a, _), (b, _)| a.cmp(b))
            .max_by(|(_, a), (_, b)| a.cmp(b))
            .map(|(alb, _)| alb)
            .unwrap();

        let son = Song {
            name: song_name.clone(),
            album: highest.clone(),
        };

        songs.insert(son, total);
    }

    songs
}

/// Returns a map with all [`Songs`][Song] corresponding to `asp` with their playcount
pub fn songs_from<Asp: HasSongs>(entries: &[SongEntry], asp: &Asp) -> HashMap<Song, usize> {
    entries
        .iter()
        .filter(|entry| asp.is_entry(entry))
        .map(Song::from)
        .counts()
}

/// Returns a map with all [`Albums`][Album] and their playcount
pub fn albums(entries: &[SongEntry]) -> HashMap<Album, usize> {
    entries.iter().map(Album::from).counts()
}

/// Returns a map with all [`Albums`][Album] corresponding to `art` with their playcount
pub fn albums_from_artist(entries: &[SongEntry], art: &Artist) -> HashMap<Album, usize> {
    entries
        .iter()
        .filter(|entry| art.is_entry(entry))
        .map(Album::from)
        .counts()
}

/// Returns a map with all [`Artists`][Artist] and their playcount
pub fn artists(entries: &[SongEntry]) -> HashMap<Artist, usize> {
    entries.iter().map(Artist::from).counts()
}

/// Returns a map with all [`Albums`][Album] corresponding to `art` with their playcount in a date range
///
/// Basically [`albums_from_artist()`] but with date functionality
///
/// # Panics
///
/// Panics if `start` is after or equal to `end`
pub fn albums_from_artist_date(
    entries: &[SongEntry],
    art: &Artist,
    start: &DateTime<Tz>,
    end: &DateTime<Tz>,
) -> HashMap<Album, usize> {
    assert!(start <= end, "Start date is after end date!");

    let (begin, stop) = find_timestamp_indexes(entries, start, end);

    entries[begin..=stop]
        .iter()
        .filter(|entry| art.is_entry(entry))
        .map(Album::from)
        .counts()
}

/// Returns a map with all [`Songs`][Song] corresponding to `asp` with their playcount in a date range
///
/// Basically [`songs_from()`] but with date functionality
///
/// # Panics
///
/// Panics if `start` is after or equal to `end`
pub fn songs_from_date<Asp: HasSongs>(
    entries: &[SongEntry],
    asp: &Asp,
    start: &DateTime<Tz>,
    end: &DateTime<Tz>,
) -> HashMap<Song, usize> {
    assert!(start <= end, "Start date is after end date!");

    let (begin, stop) = find_timestamp_indexes(entries, start, end);

    entries[begin..=stop]
        .iter()
        .filter(|entry| asp.is_entry(entry))
        .map(Song::from)
        .counts()
}

/// Counts up the plays of a single [`Music`]
pub fn plays<Asp: Music>(entries: &[SongEntry], aspect: &Asp) -> usize {
    entries
        .iter()
        .filter(|entry| aspect.is_entry(entry))
        .count()
}

/// Counts up the plays of all [`Music`] in a collection
pub fn plays_of_many<Asp: Music>(entries: &[SongEntry], aspects: &[Asp]) -> usize {
    entries
        .iter()
        .filter(|entry| aspects.iter().any(|aspect| aspect.is_entry(entry)))
        .count()
}

/// Counts up the plays of a single [`Music`] within the date range
///
/// Basically [`plays()`] but with date functionality
///
/// # Panics
///
/// Panics if `start` is after or equal to `end`
pub fn plays_date<Asp: Music>(
    entries: &[SongEntry],
    aspect: &Asp,
    start: &DateTime<Tz>,
    end: &DateTime<Tz>,
) -> usize {
    assert!(start <= end, "Start date is after end date!");

    let (begin, stop) = find_timestamp_indexes(entries, start, end);

    entries[begin..=stop]
        .iter()
        .filter(|entry| aspect.is_entry(entry))
        .count()
}

/// Counts up the plays of all [`Music`] in a collection within the date range
///
/// Basically [`plays_of_many()`] but with date functionality
///
/// # Panics
///
/// Panics if `start` is after or equal to `end`
pub fn plays_of_many_date<Asp: Music>(
    entries: &[SongEntry],
    aspects: &[Asp],
    start: &DateTime<Tz>,
    end: &DateTime<Tz>,
) -> usize {
    assert!(start <= end, "Start date is after end date!");

    let (begin, stop) = find_timestamp_indexes(entries, start, end);

    entries[begin..=stop]
        .iter()
        .filter(|entry| aspects.iter().any(|aspect| aspect.is_entry(entry)))
        .count()
}

/// Sums all plays in the given date frame
///
/// # Panics
///
/// Panics if `start` is after or equal to `end`
pub fn all_plays_date(entries: &[SongEntry], start: &DateTime<Tz>, end: &DateTime<Tz>) -> usize {
    assert!(start <= end, "Start date is after end date!");

    let (begin, stop) = find_timestamp_indexes(entries, start, end);

    entries[begin..=stop].len()
}

/// Returns the total time listened
pub fn listening_time(entries: &[SongEntry]) -> Duration {
    // sadly doesn't work bc neither chrono::Duration nor std::time::Duration implement iter::sum :))))
    // self.iter().map(|entry| entry.time_played).sum::<Duration>()
    entries
        .iter()
        .map(|entry| entry.time_played)
        .fold(Duration::milliseconds(0), |sum, dur| sum + dur)
}

/// Returns the time listened in a given date period
///
/// # Panics
///
/// Panics if `start` is after or equal to `end`
pub fn listening_time_date(
    entries: &[SongEntry],
    start: &DateTime<Tz>,
    end: &DateTime<Tz>,
) -> Duration {
    assert!(start <= end, "Start date is after end date!");

    let (begin, stop) = find_timestamp_indexes(entries, start, end);

    // sadly doesn't work bc neither chrono::Duration nor std::time::Duration implement iter::sum :))))
    // entries[begin..=stop].iter().map(|entry| entry.time_played).sum::<Duration>();
    entries[begin..=stop]
        .iter()
        .map(|entry| entry.time_played)
        .fold(Duration::milliseconds(0), |sum, dur| sum + dur)
}

/// Finds the indexes of `start` and `end` in `entries`
///
/// Uses binary search to find the indexes of the timestamps closest to `start` and `end`
/// if the exact ones are not in the dataset
fn find_timestamp_indexes(
    entries: &[SongEntry],
    start: &DateTime<Tz>,
    end: &DateTime<Tz>,
) -> (usize, usize) {
    let begin = match entries.binary_search_by(|entry| entry.timestamp.cmp(start)) {
        // timestamp from entry
        Ok(i) => i,
        // user inputted date - i because you want it to begin at the closest entry
        Err(i) if i != entries.len() => i,
        // user inputted date that's after the last entry
        Err(_) => entries.len() - 1,
    };

    let stop = match entries.binary_search_by(|entry| entry.timestamp.cmp(end)) {
        // timestamp from entry
        Ok(i) => i,
        // user inputted date - i-1 becuase i would include one entry too much
        Err(i) if i != 0 => i - 1,
        // user inputted date that's before the first entry
        Err(_) => 0,
    };

    (begin, stop)
}
