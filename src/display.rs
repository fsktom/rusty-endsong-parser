//! Module responsible for displaying the contents of endsong.json files
//! in a human-readable format (e.g. as 100 most played songs)
//! to the [`std::io::stdout`]
use itertools::Itertools;

use crate::types::Aspect;
use crate::types::AspectFull;
use crate::types::HasSongs;
use crate::types::Mode;
use crate::types::Music;
use crate::types::SongEntry;
use crate::types::{Album, Artist, Song};

use std::collections::HashMap;

/// Module containing [`display`](self) functions with
/// date functionality
pub mod date;

/// Prints the top `num` of an `asp`
///
/// * `asp` - [`Aspect::Songs`] for top songs, [`Aspect::Albums`]
///  for top albums and [`Aspect::Artists`] for top artists
/// * `num` - number of displayed top aspects.
/// Will automatically change to total number of that aspect if `num` is higher than that
/// * `sum_songs_from_different_albums` - only matters if `asp` is [`Aspect::Songs`].
/// If set to true, it will sum up the plays of
/// one song across multiple albums it may be in.
/// The album displayed in the parantheses will be the one it has the
/// highest amount of listens from.
pub fn print_top(
    entries: &[SongEntry],
    asp: Aspect,
    num: usize,
    sum_songs_from_different_albums: bool,
) {
    match asp {
        Aspect::Songs => {
            println!("=== TOP {num} SONGS ===");
            print_top_helper(gather_songs(entries, sum_songs_from_different_albums), num);
            println!();
        }
        Aspect::Albums => {
            println!("=== TOP {num} ALBUMS ===");
            print_top_helper(gather_albums(entries), num);
            println!();
        }
        Aspect::Artists => {
            println!("=== TOP {num} ARTISTS ===");
            print_top_helper(gather_artists(entries), num);
            println!();
        }
    }
}

/// Prints top songs or albums from an artist
///
/// * `mode` - [`Mode::Songs`] for top songs and [`Mode::Albums`] for top albums
/// * `artist` - the [`Artist`] you want the top songs/albums from
/// * `num` - number of displayed top songs/albums.
/// Will automatically change to total number of that aspect if `num` is higher than that
pub fn print_top_from_artist(entries: &[SongEntry], mode: Mode, artist: &Artist, num: usize) {
    match mode {
        Mode::Songs => {
            println!("=== TOP {num} SONGS FROM {artist} ===");
            print_top_helper(gather_songs_from(entries, artist), num);
            println!();
        }
        Mode::Albums => {
            println!("=== TOP {num} ALBUMS FROM {artist} ===");
            print_top_helper(gather_albums_from_artist(entries, artist), num);
            println!();
        }
    }
}

/// Prints top songs from an album
///
/// * `album` - the [`Album`] you want the top songs from
/// * `num` - number of displayed top songs.
/// Will automatically change to total number of songs from that album if `num` is higher than that
pub fn print_top_from_album(entries: &[SongEntry], album: &Album, num: usize) {
    println!("=== TOP {num} SONGS FROM {album} ===");
    print_top_helper(gather_songs_from(entries, album), num);
    println!();
}

/// Used by [`print_top()`]
fn print_top_helper<Asp: Music>(music_dict: HashMap<Asp, usize>, num: usize) {
    // https://stackoverflow.com/q/34555837/6694963
    let mut music_vec: Vec<(Asp, usize)> = music_dict.into_iter().collect();
    let length = music_vec.len();

    // primary sorting: sort by plays
    music_vec.sort_by(|a, b| b.1.cmp(&a.1));

    // secondary sorting: if plays are equal -> sort A->Z
    let mut alphabetical: Vec<(Asp, usize)> = Vec::with_capacity(length);
    let mut same_plays: Vec<(Asp, usize)> = vec![music_vec.first().unwrap().to_owned()];
    for el in music_vec {
        let first = same_plays.first().unwrap();
        // ignore first element of list (cause it's already in same_plays)
        if el.0 == first.0 {
            continue;
        }

        // if the plays of the new element are equal to the one(s) already
        // in same_plays -> add element to same_plays
        if el.1 == first.1 {
            same_plays.push(el);
        // if they're not equal, that means same_plays can be sorted alphabetically
        // bc all elements have same num of plays
        // and then added to the new vector
        } else {
            same_plays.sort_by(|a, b| a.0.cmp(&b.0));
            alphabetical.append(&mut same_plays);
            same_plays = vec![el];
        }
    }
    // final step bc possible that last element has same num of plays
    // as the second-to-last element
    same_plays.sort_by(|a, b| a.0.cmp(&b.0));
    alphabetical.append(&mut same_plays);

    // something must have gone wrong if this fails
    assert!(alphabetical.len() == length);

    // if the number of unique aspects is lower than the parsed num
    let max_num: usize = if length < num { length } else { num };

    for (i, (asp, plays)) in alphabetical.iter().enumerate() {
        println!(
            "{}: {} | {} plays",
            leading_whitespace(i + 1, max_num),
            asp,
            plays
        );

        if i + 1 == max_num {
            break;
        }
    }
}

/// Formats `1` to ` #1` if user wishes for Top 10
/// or to `  #1` if Top 100 etc.
///
/// # Arguments
/// * `num` - position of the [`AspectFull`], must be >0
/// * `max_num` - the highest position you want to display,
/// must be >0 and should be >=`num`
///
/// # Panics
///
/// Panics if `num` or `max_num` is 0
///
/// # Examples
/// ```
/// use rusty_endsong_parser::display::leading_whitespace;
/// assert_eq!(leading_whitespace(7usize, 100usize), String::from("  #7"));
/// assert_eq!(leading_whitespace(7usize, 1000usize), String::from("   #7"));
/// ```
pub fn leading_whitespace(num: usize, max_num: usize) -> String {
    assert!(num > 0);
    assert!(max_num > 0);
    // https://github.com/Filip-Tomasko/endsong-parser-python/blob/main/src/endsong_parser.py#L551-L578
    let mut order_format = String::new();

    let mut num_of_zero = max_num.ilog10();
    let digits = num.ilog10() + 1;

    loop {
        if num_of_zero == 0 {
            break;
        }
        if digits <= num_of_zero {
            order_format += " ";
        }
        num_of_zero -= 1;
    }

    format!("{order_format}#{num}")
}

/// Basically [`Song`] but without the [`album`][Song] field
///
/// used in [`print_top()`] if `sum_songs_from_different_albums` is set to true
/// and in [`find_song()`]
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
struct SongJustArtist {
    /// Name of the song
    name: String,
    /// Artist of the song
    artist: Artist,
}
impl From<&Song> for SongJustArtist {
    fn from(song: &Song) -> SongJustArtist {
        SongJustArtist {
            name: song.name.to_string(),
            artist: song.album.artist.clone(),
        }
    }
}

/// Returns a map with all [`Songs`][Song] and their playcount
///
/// `sum_songs_from_different_albums` - with `true` it will summarize the plays
/// of songs if their name and artist is the same;
/// with `false` it will also take into account the album the song is in
///
/// It matters because oftentimes the same song will be in many albums (or singles).
/// But it's still case-sensitive!
fn gather_songs(
    entries: &[SongEntry],
    sum_songs_from_different_albums: bool,
) -> HashMap<Song, usize> {
    let mut songs = entries.iter().map(Song::from).counts();
    if !sum_songs_from_different_albums {
        return songs;
    }

    // to know which album the song had highest amount of plays from
    // that album will be then displayed in () after the song name
    // but the number of plays that will be displayed will be a sum of
    // the plays from all albums
    let mut changed: HashMap<SongJustArtist, HashMap<Album, usize>> = HashMap::new();
    for (song, plays_song) in &songs {
        let song_just_artist = SongJustArtist::from(song);

        changed
            .entry(song_just_artist)
            .or_insert_with(HashMap::new)
            .insert(song.album.clone(), *plays_song);
    }

    // required because only one version (i.e. album) of the song should be saved
    songs.clear();

    for (song_just_artist, albs) in &changed {
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
            name: song_just_artist.name.clone(),
            album: highest.clone(),
        };

        songs.insert(son, total);
    }

    songs
}

/// Returns a map with all [`Songs`][Song] corresponding to `asp` with their playcount
fn gather_songs_from<Asp: HasSongs>(entries: &[SongEntry], asp: &Asp) -> HashMap<Song, usize> {
    entries
        .iter()
        .filter(|entry| asp.is_entry(entry))
        .map(Song::from)
        .counts()
}

/// Returns a map with all [`Albums`][Album] and their playcount
fn gather_albums(entries: &[SongEntry]) -> HashMap<Album, usize> {
    entries.iter().map(Album::from).counts()
}

/// Returns a map with all [`Albums`][Album] corresponding to `art` with their playcount
fn gather_albums_from_artist(entries: &[SongEntry], art: &Artist) -> HashMap<Album, usize> {
    entries
        .iter()
        .filter(|entry| art.is_entry(entry))
        .map(Album::from)
        .counts()
}

/// Returns a map with all [`Artists`][Artist] and their playcount
fn gather_artists(entries: &[SongEntry]) -> HashMap<Artist, usize> {
    entries.iter().map(Artist::from).counts()
}

/// Prints a specfic aspect
///
/// * `asp` - the aspect you want informationa about containing the
/// relevant struct
pub fn print_aspect(entries: &[SongEntry], asp: &AspectFull) {
    match *asp {
        AspectFull::Artist(art) => {
            println!("=== {} | {} plays ===", art, gather_plays(entries, art));
            // TODO! currently print_artist uses the whole time for num of plays!!!
            // e.g. printing Alestorm between 2022-01-01 and 2022-07-01
            // on only `endsong_0.json`
            // will print:
            // === Alestorm between 2022-01-01CET and 2022-07-01CEST | 1 plays ===
            // --- Alestorm - Sunset On The Golden Age | 3 plays ---
            // #1: Alestorm - Drink (Sunset On The Golden Age) | 3 plays

            print_artist(entries, &gather_albums_from_artist(entries, art));
        }
        AspectFull::Album(alb) => {
            println!("=== {} | {} plays ===", alb, gather_plays(entries, alb));
            // TODO! currently print_album uses the whole time for num of plays!!!
            print_album(&gather_songs_from(entries, alb));
        }
        AspectFull::Song(son) => {
            println!("{} | {} plays", son, gather_plays(entries, son));
        }
    }
}

/// Counts up the plays of a single [`Music`]
fn gather_plays<Asp: Music>(entries: &[SongEntry], aspect: &Asp) -> usize {
    entries
        .iter()
        .filter(|entry| aspect.is_entry(entry))
        .count()
}

/// Prints each [`Album`] of `albums` with the playcount
fn print_artist(entries: &[SongEntry], albums: &HashMap<Album, usize>) {
    let mut albums_vec: Vec<(&Album, &usize)> = albums.iter().collect();
    albums_vec.sort_by(|a, b| b.1.cmp(a.1));

    for (alb, plays) in albums_vec {
        println!("--- {alb} | {plays} plays ---");
        print_album(&gather_songs_from(entries, alb));
    }
}

/// Prints each [`Song`] of `songs` with the playcount
fn print_album(songs: &HashMap<Song, usize>) {
    let mut songs_vec: Vec<(&Song, &usize)> = songs.iter().collect();
    songs_vec.sort_by(|a, b| b.1.cmp(a.1));

    for (i, (song, plays)) in songs_vec.iter().enumerate() {
        println!(
            "{}: {song} | {plays} plays",
            leading_whitespace(i + 1, songs_vec.len())
        );
    }
}

/// Searches the entries for if the given artist exists in the dataset
///
/// Case-insensitive and returns the [`Artist`] with proper capitalization
/// (i.e. the capitalization of the first entry it finds)
///
/// See #2 <https://github.com/fsktom/rusty-endsong-parser/issues/2>
pub fn find_artist(entries: &[SongEntry], artist_name: &str) -> Option<Artist> {
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
pub fn find_album(entries: &[SongEntry], album_name: &str, artist_name: &str) -> Option<Album> {
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
pub fn find_song_from_album(
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
pub fn find_song(entries: &[SongEntry], song_name: &str, artist_name: &str) -> Option<Vec<Song>> {
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
pub fn find_songs_from_album(entries: &[SongEntry], album: &Album) -> Vec<Song> {
    entries
        .iter()
        .filter(|entry| album.is_entry(entry))
        .unique()
        .map(Song::from)
        .collect_vec()
}

/// Prints a specfic aspect within a date frame
///
/// Basically [`print_aspect()`] but with date limitations
///
/// * `asp` - the aspect you want informationa about containing the
/// relevant struct
///
/// Wrapper around [`date::print_aspect()`]
pub fn print_aspect_date(
    entries: &[SongEntry],
    asp: &AspectFull,
    start: &chrono::DateTime<chrono_tz::Tz>,
    end: &chrono::DateTime<chrono_tz::Tz>,
) {
    date::print_aspect(entries, asp, start, end);
}

/// Prints the total time played
pub fn print_time_played(entries: &crate::types::SongEntries) {
    let duration = entries.total_listening_time();

    println!(
        "You've spent {} days - or {} hours - or {} minutes listening to music!",
        &duration.num_days(),
        &duration.num_hours(),
        &duration.num_minutes()
    );
}

/// Prints the time played in a duration
///
/// Basically [`print_time_played()`] but with date limitation
///
/// Wrapper around [`date::print_time_played()`]
pub fn print_time_played_date(
    entries: &crate::types::SongEntries,
    start: &chrono::DateTime<chrono_tz::Tz>,
    end: &chrono::DateTime<chrono_tz::Tz>,
) {
    date::print_time_played(entries, start, end);
}

// https://doc.rust-lang.org/book/ch11-03-test-organization.html#unit-tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn order_format() {
        assert_eq!(leading_whitespace(3usize, 100usize), String::from("  #3"));
        assert_eq!(leading_whitespace(3usize, 1000usize), String::from("   #3"));
        assert_eq!(leading_whitespace(3usize, 5692usize), String::from("   #3"));
    }

    #[test]
    #[should_panic]
    fn order_format_zero() {
        leading_whitespace(0usize, 100usize);
        leading_whitespace(1usize, 0usize);
    }

    #[test]
    fn find_aspect() {
        // MAYBE RATHER INTEGRATION TEST THAN UNIT TEST?!
        let paths = vec![format!(
            "{}/stuff/example_endsong/endsong_0.json",
            std::env::current_dir().unwrap().display()
        )];
        let entries = crate::types::SongEntries::new(&paths).unwrap();

        assert_eq!(
            find_artist(&entries, "Theocracy").unwrap(),
            Artist::new("Theocracy")
        );
        assert!(entries.find().artist("Powerwolf").is_none());
    }
}
