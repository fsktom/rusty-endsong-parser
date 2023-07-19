use std::collections::HashMap;

use chrono::DateTime;
use chrono_tz::Tz;

use crate::types::AspectFull;
use crate::types::Music;
use crate::types::SongEntry;
use crate::types::{Album, Artist, Song};

/// Prints the time played in a date range
///
/// # Panics
///
/// Panics if `start` is after or equal to `end`
#[allow(clippy::cast_precision_loss)]
pub fn print_time_played(
    entries: &crate::types::SongEntries,
    start: &DateTime<Tz>,
    end: &DateTime<Tz>,
) {
    assert!(start <= end, "Start date is after end date!");
    let duration = entries.listening_time(start, end);
    let period = *end - *start;

    println!(
        "You've spent {} days ({:.2}%) ({} hours / {} minutes) listening to music between {} and {} ({} days à {} plays/day & {} hours/day)!",
        &duration.num_days(),
        ((duration.num_minutes() as f64) / (period.num_minutes() as f64)) * 100.0,
        &duration.num_hours(),
        &duration.num_minutes(),
        start.date_naive(),
        end.date_naive(),
        period.num_days(),
        sum_plays(entries, start, end) as i64 / period.num_days(),
        duration.num_hours() / period.num_days(),
    );
}

/// Prints a specfic aspect
///
/// Basically [`super::print_aspect()`] but with date limitations
///
/// * `asp` - the aspect you want informationa about containing the
/// relevant struct
///
/// # Panics
///
/// Panics if `start` is after or equal to `end`
pub fn print_aspect(
    entries: &[SongEntry],
    asp: &AspectFull,
    start: &DateTime<Tz>,
    end: &DateTime<Tz>,
) {
    assert!(start <= end, "Start date is after end date!");
    match *asp {
        AspectFull::Artist(art) => {
            println!(
                "=== {} between {} and {} | {} plays ===",
                art,
                start.date_naive(),
                end.date_naive(),
                gather_plays(entries, art, start, end)
            );
            print_artist(
                entries,
                &gather_albums_with_artist(entries, art, start, end),
                start,
                end,
            );
        }
        AspectFull::Album(alb) => {
            println!(
                "=== {} between {} and {} | {} plays ===",
                alb,
                start.date_naive(),
                end.date_naive(),
                gather_plays(entries, alb, start, end)
            );
            super::print_album(&gather_songs_with_album(entries, alb, start, end));
        }
        AspectFull::Song(son) => {
            println!(
                "{} between {} and {} | {} plays",
                son,
                start.date_naive(),
                end.date_naive(),
                gather_plays(entries, son, start, end)
            );
        }
    }
}

/// Prints each [`Album`] of `albums` with the playcount in the date range
///
/// # Panics
///
/// Panics if `start` is after or equal to `end`
fn print_artist(
    entries: &[SongEntry],
    albums: &HashMap<Album, u32>,
    start: &DateTime<Tz>,
    end: &DateTime<Tz>,
) {
    assert!(start <= end, "Start date is after end date!");
    let mut albums_vec: Vec<(&Album, &u32)> = albums.iter().collect();
    albums_vec.sort_by(|a, b| b.1.cmp(a.1));

    for (alb, plays) in albums_vec {
        println!("--- {alb} | {plays} plays ---");
        super::print_album(&gather_songs_with_album(entries, alb, start, end));
    }
}

/// Counts up the plays of a single [`Music`] within the date range
///
/// Basically [`super::gather_plays()`] but with date functionality
///
/// # Panics
///
/// Panics if `start` is after or equal to `end`
pub fn gather_plays<Asp: Music>(
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

/// Returns a map with all [`Albums`][Album] corresponding to `art` with their playcount in a date range
///
/// Basically [`super::gather_albums_with_artist()`] but with date functionality
///
/// # Panics
///
/// Panics if `start` is after or equal to `end`
fn gather_albums_with_artist(
    entries: &[SongEntry],
    art: &Artist,
    start: &DateTime<Tz>,
    end: &DateTime<Tz>,
) -> HashMap<Album, u32> {
    assert!(start <= end, "Start date is after end date!");
    let mut albums: HashMap<Album, u32> = HashMap::new();

    let (begin, stop) = find_timestamp_indexes(entries, start, end);

    for album in entries[begin..=stop]
        .iter()
        .filter(|entry| art.is_entry(entry))
        .map(Album::from)
    {
        *albums.entry(album).or_insert(0) += 1;
    }

    albums
}

/// Returns a map with all [`Songs`][Song] corresponding to `alb` with their playcount in a date range
///
/// Basically [`super::gather_songs_with_album()`] but with date functionality
///
/// # Panics
///
/// Panics if `start` is after or equal to `end`
fn gather_songs_with_album(
    entries: &[SongEntry],
    alb: &Album,
    start: &DateTime<Tz>,
    end: &DateTime<Tz>,
) -> HashMap<Song, u32> {
    assert!(start <= end, "Start date is after end date!");
    let mut songs: HashMap<Song, u32> = HashMap::new();

    let (begin, stop) = find_timestamp_indexes(entries, start, end);

    for song in entries[begin..=stop]
        .iter()
        .filter(|entry| alb.is_entry(entry))
        .map(Song::from)
    {
        *songs.entry(song).or_insert(0) += 1;
    }

    songs
}

/// Sums all plays in the given date frame
///
/// # Panics
///
/// Panics if `start` is after or equal to `end`
pub fn sum_plays(entries: &[SongEntry], start: &DateTime<Tz>, end: &DateTime<Tz>) -> usize {
    assert!(start <= end, "Start date is after end date!");

    let (begin, stop) = find_timestamp_indexes(entries, start, end);

    entries[begin..=stop].len()
}

/// Finds the indexes of `start` and `end` in `entries`
///
/// Uses binary search to find the indexes of the timestamps closest to `start` and `end`
/// if the exact ones are not in the dataset
pub fn find_timestamp_indexes(
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
