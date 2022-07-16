use std::collections::HashMap;

use chrono::DateTime;
use chrono_tz::Tz;

use crate::types::AspectFull;
use crate::types::SongEntry;
use crate::types::{Album, Artist, Song};

use super::{print_album, print_artist};
use super::{AlbumPlays, ArtistPlays, SongPlays};

/// Prints a specfic aspect
///
/// Basically [print_aspect()] but with date limitations
///
/// * `asp` - the aspect you want informationa about containing the
/// relevant struct
pub fn print_aspect_date(
    entries: &Vec<SongEntry>,
    asp: AspectFull,
    start: &DateTime<Tz>,
    end: &DateTime<Tz>,
) {
    match asp {
        AspectFull::Artist(art) => {
            println!(
                "=== {} between {} and {} | {} plays ===",
                art,
                start.date(),
                end.date(),
                gather_artist_date(entries, art, start, end).1
            );
            print_artist(
                entries,
                gather_albums_with_artist_date(entries, art, start, end),
            );
        }
        AspectFull::Album(alb) => {
            println!(
                "=== {} between {} and {} | {} plays ===",
                alb,
                start.date(),
                end.date(),
                gather_album_date(entries, alb, start, end).1
            );
            print_album(gather_songs_with_album_date(entries, alb, start, end));
        }
        AspectFull::Song(son) => {
            let son = gather_song_date(entries, son, start, end);
            println!("{} between {} and {} | {} plays", son.0, start, end, son.1);
        }
    }
}

/// Counts up the plays of a single artist within a date frame
///
/// Basically [gather_artist()] but with date functionality
fn gather_artist_date(
    entries: &Vec<SongEntry>,
    art: &Artist,
    start: &DateTime<Tz>,
    end: &DateTime<Tz>,
) -> ArtistPlays {
    let mut artist_asp = ArtistPlays(art.clone(), 0);

    for entry in entries {
        let artist = Artist::new(entry.artist.clone());

        if entry.timestamp.ge(start) && entry.timestamp.le(end) && artist == *art {
            artist_asp.1 += 1;
        }
    }

    artist_asp
}

/// Counts up the plays of a single album within a date frame
///
/// Basically [gather_album()] but with date functionality
fn gather_album_date(
    entries: &Vec<SongEntry>,
    alb: &Album,
    start: &DateTime<Tz>,
    end: &DateTime<Tz>,
) -> AlbumPlays {
    let mut album_asp = AlbumPlays(alb.clone(), 0);

    for entry in entries {
        let album = Album::new(entry.album.clone(), entry.artist.clone());

        if entry.timestamp.ge(start) && entry.timestamp.le(end) && album == *alb {
            album_asp.1 += 1;
        }
    }

    album_asp
}

/// Counts up the plays of a single song within a date frame
///
/// Basically [gather_song()] but with date functionality
fn gather_song_date(
    entries: &Vec<SongEntry>,
    son: &Song,
    start: &DateTime<Tz>,
    end: &DateTime<Tz>,
) -> SongPlays {
    let mut song_asp = SongPlays(son.clone(), 0);

    for entry in entries {
        let song = Song::new(
            entry.track.clone(),
            entry.album.clone(),
            entry.artist.clone(),
        );

        if entry.timestamp.ge(start) && entry.timestamp.le(end) && song == *son {
            song_asp.1 += 1;
        }
    }

    song_asp
}

/// Used by [print_aspect_date()]
///
/// Basically [gather_albums_with_artist()] but with date functionality
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

/// Used by [print_aspect_date()]
///
/// Basically [gather_songs_with_album()] but with date functionality
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
