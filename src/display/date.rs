use std::collections::HashMap;

use chrono::DateTime;
use chrono_tz::Tz;

use crate::types::AspectFull;
use crate::types::IsBetween;
use crate::types::Music;
use crate::types::SongEntry;
use crate::types::{Album, Artist, Song};

/// Prints the time played in a date range
#[allow(clippy::cast_precision_loss)]
pub fn print_time_played(
    entries: &crate::types::SongEntries,
    start: &DateTime<Tz>,
    end: &DateTime<Tz>,
) {
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
pub fn print_aspect(
    entries: &[SongEntry],
    asp: &AspectFull,
    start: &DateTime<Tz>,
    end: &DateTime<Tz>,
) {
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

/// Used by [`print_aspect()`]
fn print_artist(
    entries: &[SongEntry],
    albums: &HashMap<Album, u32>,
    start: &DateTime<Tz>,
    end: &DateTime<Tz>,
) {
    let mut albums_vec: Vec<(&Album, &u32)> = albums.iter().collect();
    albums_vec.sort_by(|a, b| b.1.cmp(a.1));

    for (alb, plays) in albums_vec {
        println!("--- {alb} | {plays} plays ---");
        super::print_album(&gather_songs_with_album(entries, alb, start, end));
    }
}

/// Counts up the plays of a single [`Music`] within the date range
///
/// Basically [`display::gather_plays()`][super::gather_plays()] but with date functionality
pub fn gather_plays<Asp: Music>(
    entries: &[SongEntry],
    aspect: &Asp,
    start: &DateTime<Tz>,
    end: &DateTime<Tz>,
) -> usize {
    let begin = entries
        .binary_search_by(|entry| entry.timestamp.cmp(start))
        .unwrap();

    let stop = entries
        .binary_search_by(|entry| entry.timestamp.cmp(end))
        .unwrap();

    entries[begin..=stop]
        .iter()
        .filter(|entry| aspect.is_entry(entry))
        .count()
}

/// Used by [`print_aspect()`]
///
/// Basically [`super::gather_albums_with_artist()`] but with date functionality
fn gather_albums_with_artist(
    entries: &[SongEntry],
    art: &Artist,
    start: &DateTime<Tz>,
    end: &DateTime<Tz>,
) -> HashMap<Album, u32> {
    let mut albums: HashMap<Album, u32> = HashMap::new();

    for entry in entries {
        if art.is_entry(entry) && entry.timestamp.is_between(start, end) {
            let alb = Album::new(&entry.album, &entry.artist);
            *albums.entry(alb).or_insert(0) += 1;
        }
    }

    albums
}

/// Used by [`print_aspect()`] and [`print_artist()`]
///
/// Basically [`super::gather_songs_with_album()`] but with date functionality
fn gather_songs_with_album(
    entries: &[SongEntry],
    alb: &Album,
    start: &DateTime<Tz>,
    end: &DateTime<Tz>,
) -> HashMap<Song, u32> {
    let mut songs: HashMap<Song, u32> = HashMap::new();

    for entry in entries {
        if alb.is_entry(entry) && entry.timestamp.is_between(start, end) {
            let song = Song::new(&entry.track, &entry.album, &entry.artist);
            *songs.entry(song).or_insert(0) += 1;
        }
    }

    songs
}

/// Sums all plays in the given date frame
pub fn sum_plays(entries: &[SongEntry], start: &DateTime<Tz>, end: &DateTime<Tz>) -> usize {
    let begin = entries
        .binary_search_by(|entry| entry.timestamp.cmp(start))
        .unwrap_or(0);
    // unwrap_or because it may fail when you input a date that is before the first entry
    // or after the last entry I think?
    let stop = entries
        .binary_search_by(|entry| entry.timestamp.cmp(end))
        .unwrap_or(entries.len() - 1);

    entries[begin..=stop].len()
}
