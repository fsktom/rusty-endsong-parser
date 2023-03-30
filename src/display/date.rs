use std::collections::HashMap;

use chrono::DateTime;
use chrono_tz::Tz;

use crate::types::AspectFull;
use crate::types::Music;
use crate::types::SongEntry;
use crate::types::{Album, Artist, Song};

use super::print_album;

/// Prints a specfic aspect
///
/// Basically [`super::print_aspect()`] but with date limitations
///
/// * `asp` - the aspect you want informationa about containing the
/// relevant struct
pub fn print_aspect(
    entries: &Vec<SongEntry>,
    asp: &AspectFull,
    start: &DateTime<Tz>,
    end: &DateTime<Tz>,
) {
    match asp {
        AspectFull::Artist(art) => {
            println!(
                "=== {} between {} and {} | {} plays ===",
                art,
                start.date_naive(),
                end.date_naive(),
                gather_plays(entries, *art, start, end)
            );
            print_artist(
                entries,
                &gather_albums_with_artist_date(entries, art, start, end),
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
                gather_plays(entries, *alb, start, end)
            );
            print_album(&gather_songs_with_album_date(entries, alb, start, end));
        }
        AspectFull::Song(son) => {
            println!(
                "{} between {} and {} | {} plays",
                son,
                start.date_naive(),
                end.date_naive(),
                gather_plays(entries, *son, start, end)
            );
        }
    }
}

/// Used by [`print_aspect()`]
fn print_artist(
    entries: &Vec<SongEntry>,
    artist: &HashMap<Album, u32>,
    start: &DateTime<Tz>,
    end: &DateTime<Tz>,
) {
    let mut artist_vec: Vec<(&Album, &u32)> = artist.iter().collect();
    artist_vec.sort_by(|a, b| b.1.cmp(a.1));

    for i in 0..artist_vec.len() {
        let alb = artist_vec.get(i).unwrap().0;
        let mus = gather_songs_with_album_date(entries, alb, start, end);
        // calling gather_album here is unnecessary work
        // it should add up the total plays somehwere else
        println!(
            "--- {} | {} plays ---",
            alb,
            gather_plays(entries, alb, start, end)
        );
        print_album(&mus);
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
    entries
        .iter()
        .filter(|entry| aspect.is_entry(entry) && is_between(&entry.timestamp, start, end))
        .count()
}

/// Used by [`print_aspect()`]
///
/// Basically [`super::gather_albums_with_artist()`] but with date functionality
fn gather_albums_with_artist_date(
    entries: &Vec<SongEntry>,
    art: &Artist,
    start: &DateTime<Tz>,
    end: &DateTime<Tz>,
) -> HashMap<Album, u32> {
    let mut albums: HashMap<Album, u32> = HashMap::new();

    for entry in entries {
        if Artist::new(entry.artist.clone()) != *art {
            continue;
        }
        if entry.timestamp.ge(start) && entry.timestamp.le(end) {
            let album = Album::new(entry.album.clone(), entry.artist.clone());
            *albums.entry(album).or_insert(0) += 1;
        }
    }

    albums
}

/// Used by [`print_aspect()`]
///
/// Basically [`super::gather_songs_with_album()`] but with date functionality
fn gather_songs_with_album_date(
    entries: &Vec<SongEntry>,
    alb: &Album,
    start: &DateTime<Tz>,
    end: &DateTime<Tz>,
) -> HashMap<Song, u32> {
    let mut songs: HashMap<Song, u32> = HashMap::new();

    for entry in entries {
        if Album::new(entry.album.clone(), entry.artist.clone()) != *alb {
            continue;
        }

        if entry.timestamp.ge(start) && entry.timestamp.le(end) {
            let song = Song::new(
                entry.track.clone(),
                entry.album.clone(),
                entry.artist.clone(),
            );

            *songs.entry(song).or_insert(0) += 1;
        }
    }

    songs
}

/// Sums all plays in the given date frame
pub fn sum_plays(entries: &[SongEntry], start: &DateTime<Tz>, end: &DateTime<Tz>) -> usize {
    entries
        .iter()
        .filter(|entry| is_between(&entry.timestamp, start, end))
        .count()
}

/// Checks if the given date is between (or equal) to the other two dates
fn is_between(date: &DateTime<Tz>, start: &DateTime<Tz>, end: &DateTime<Tz>) -> bool {
    date.ge(start) && date.le(end)
}
