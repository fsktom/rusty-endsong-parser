//! Module responsible for displaying the contents of endsong.json files
//! in a human-readable format (e.g. as 100 most played songs)
//! to the [`std::io::stdout`]
use crate::types::Aspect;
use crate::types::AspectFull;
use crate::types::Mode;
use crate::types::Music;
use crate::types::NotFoundError;
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
    asp: &Aspect,
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
/// * `asp` - [`Aspect::Songs`] for top songs and [`Aspect::Albums`] for top albums
/// * `artist` - the [`Artist`] you want the top songs/albums from
/// * `num` - number of displayed top aspects. Will automatically change to total number of that aspect if `num` is higher than that
pub fn print_top_from_artist(entries: &[SongEntry], mode: &Mode, artist: &Artist, num: usize) {
    match mode {
        Mode::Songs => {
            println!("=== TOP {num} SONGS FROM {artist} ===");
            print_top_helper(gather_songs_with_artist(entries, artist), num);
            println!();
        }
        Mode::Albums => {
            println!("=== TOP {num} ALBUMS FROM {artist} ===");
            print_top_helper(gather_albums_with_artist(entries, artist), num);
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
    print_top_helper(gather_songs_with_album(entries, album), num);
    println!();
}

/// Used by [`print_top()`]
fn print_top_helper<Asp: Music>(music_dict: HashMap<Asp, u32>, num: usize) {
    // https://stackoverflow.com/q/34555837/6694963
    let mut music_vec: Vec<(Asp, u32)> = music_dict.into_iter().collect();
    let length = music_vec.len();

    // primary sorting: sort by plays
    music_vec.sort_by(|a, b| b.1.cmp(&a.1));

    // secondary sorting: if plays are equal -> sort A->Z
    let mut alphabetical: Vec<(Asp, u32)> = Vec::with_capacity(length);
    let mut same_plays: Vec<(Asp, u32)> = vec![music_vec.first().unwrap().to_owned()];
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
/// # Examples
/// ```
/// assert_eq!(leading_whitespace(7usize, 100usize), String::from("  #7"));
/// assert_eq!(leading_whitespace(7usize, 1000usize), String::from("   #7"));
/// ```
fn leading_whitespace(num: usize, max_num: usize) -> String {
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
impl SongJustArtist {
    /// Creates an instance of [`SongJustArtist`]
    fn new<S: Into<String>>(song_name: S, artist_name: S) -> SongJustArtist {
        SongJustArtist {
            name: song_name.into(),
            artist: Artist::new(artist_name),
        }
    }
}
impl From<&Song> for SongJustArtist {
    fn from(song: &Song) -> SongJustArtist {
        SongJustArtist {
            name: song.name.to_string(),
            artist: song.album.artist.clone(),
        }
    }
}

/// Used by [`print_top_helper()`]
#[allow(clippy::needless_range_loop)]
fn gather_songs(
    entries: &[SongEntry],
    sum_songs_from_different_albums: bool,
) -> HashMap<Song, u32> {
    let mut songs: HashMap<Song, u32> = HashMap::new();

    for entry in entries {
        let song = Song::new(&entry.track, &entry.album, &entry.artist);

        // either create new field with value 0 (and add 1 to it)
        // or if a field with that key already exists,
        // add one play to it
        *songs.entry(song).or_insert(0) += 1;
    }

    if sum_songs_from_different_albums {
        /// Tuple struct containing an Album with the amount of plays
        #[derive(PartialEq, Eq, Hash, Debug, Clone)]
        struct AlbumPlays(Album, u32);

        /// Contains the name of the song and
        /// a vector containg all the albums this song is in
        #[derive(PartialEq, Eq, Hash, Debug, Clone)]
        struct SongAlbums {
            /// Name of the song
            name: String,
            /// Vector with the albums the song is in with
            /// the amount of plays in each album
            albums: Vec<AlbumPlays>,
        }

        let mut songs_artist: HashMap<SongJustArtist, u32> = HashMap::new();

        // to know which album the song had highest amount of plays from
        // that album will be then displayed in () after the song name
        // but the number of plays that will be displayed will be a sum of
        // the plays from all albums
        let mut changed: HashMap<SongJustArtist, SongAlbums> = HashMap::new();

        for (k, v) in &songs {
            let song_just_artist = SongJustArtist::from(k);

            if let Some(plays) = songs_artist.get(&song_just_artist) {
                // if it finds something it means that the song
                // only from a different album already exists

                *songs_artist.entry(song_just_artist.clone()).or_insert(0) += *plays;

                let temp = changed.get_mut(&song_just_artist).unwrap();
                temp.albums.push(AlbumPlays(k.album.clone(), *v));
            } else {
                // if it doesn't find anything, it's the first appearance of that song
                songs_artist.insert(song_just_artist.clone(), *v);
                let salb = SongAlbums {
                    name: k.name.clone(),
                    albums: vec![AlbumPlays(k.album.clone(), *v)],
                };
                changed.insert(song_just_artist.clone(), salb);
            }
        }

        for (k, v) in &changed {
            let albs = &v.albums;

            // the first album will be taken if both have
            // the same number of plays
            let mut total: u32 = 0;
            let highest: &AlbumPlays = {
                let mut plays = 0;
                for alb in 0..albs.len() {
                    if albs[alb].1 > plays {
                        plays = albs[alb].1;
                    }
                    total += albs[alb].1;
                }
                &albs[0]
            };

            let son = Song {
                name: k.name.clone(),
                album: highest.0.clone(),
            };

            songs.insert(son, total);
        }
    }
    songs
}

/// Used by [`print_top_helper()`]
fn gather_songs_with_artist(entries: &[SongEntry], art: &Artist) -> HashMap<Song, u32> {
    let mut songs: HashMap<Song, u32> = HashMap::new();

    for entry in entries {
        if art.is_entry(entry) {
            let song = Song::new(&entry.track, &entry.album, &entry.artist);

            *songs.entry(song).or_insert(0) += 1;
        }
    }

    songs
}

/// Used by [`print_top_helper()`], [`print_aspect()`] and [`print_album()`]
fn gather_songs_with_album(entries: &[SongEntry], alb: &Album) -> HashMap<Song, u32> {
    let mut songs: HashMap<Song, u32> = HashMap::new();

    for entry in entries {
        if alb.is_entry(entry) {
            let song = Song::new(&entry.track, &entry.album, &entry.artist);

            *songs.entry(song).or_insert(0) += 1;
        }
    }

    songs
}

/// Used by [`print_top_helper()`]
fn gather_albums(entries: &[SongEntry]) -> HashMap<Album, u32> {
    let mut albums: HashMap<Album, u32> = HashMap::new();

    for entry in entries {
        let album = Album::new(&entry.album, &entry.artist);

        *albums.entry(album).or_insert(0) += 1;
    }

    albums
}

/// Used by [`print_top_helper()`] and [`print_aspect()`]
fn gather_albums_with_artist(entries: &[SongEntry], art: &Artist) -> HashMap<Album, u32> {
    let mut albums: HashMap<Album, u32> = HashMap::new();

    for entry in entries {
        if art.is_entry(entry) {
            let album = Album::new(&entry.album, &entry.artist);
            *albums.entry(album).or_insert(0) += 1;
        }
    }

    albums
}

/// Used by [`print_top_helper()`]
fn gather_artists(entries: &[SongEntry]) -> HashMap<Artist, u32> {
    let mut artists: HashMap<Artist, u32> = HashMap::new();

    for entry in entries {
        let artist = Artist::new(&entry.artist);

        *artists.entry(artist).or_insert(0) += 1;
    }

    artists
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

            print_artist(entries, &gather_albums_with_artist(entries, art));
        }
        AspectFull::Album(alb) => {
            println!("=== {} | {} plays ===", alb, gather_plays(entries, alb));
            // TODO! currently print_album uses the whole time for num of plays!!!
            print_album(&gather_songs_with_album(entries, alb));
        }
        AspectFull::Song(son) => {
            println!("{} | {} plays", son, gather_plays(entries, son));
        }
    }
}

/// Counts up the plays of a [`Music`] instance
fn gather_plays<Asp: Music>(entries: &[SongEntry], aspect: &Asp) -> usize {
    entries
        .iter()
        .filter(|entry| aspect.is_entry(entry))
        .count()
}

/// Used by [`print_aspect()`]
fn print_artist(entries: &[SongEntry], albums: &HashMap<Album, u32>) {
    let mut albums_vec: Vec<(&Album, &u32)> = albums.iter().collect();
    albums_vec.sort_by(|a, b| b.1.cmp(a.1));

    for (alb, plays) in albums_vec {
        println!("--- {alb} | {plays} plays ---");
        print_album(&gather_songs_with_album(entries, alb));
    }
}

/// Used by [`print_aspect()`]
fn print_album(songs: &HashMap<Song, u32>) {
    let mut songs_vec: Vec<(&Song, &u32)> = songs.iter().collect();
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
/// Wrapped by [`crate::types::Find::artist()`]
///
/// # Errors
///
/// This function will return an [`Err`] with [`NotFoundError::Artist`]
/// if it cannot find an artist with the given name
pub fn find_artist(entries: &[SongEntry], artist_name: &str) -> Result<Artist, NotFoundError> {
    let usr_artist = Artist::new(artist_name.to_lowercase());

    for entry in entries {
        // .to_lowercase() so that user input capitalization doesn't matter
        if entry.artist.to_lowercase().eq(&usr_artist.name) {
            return Ok(Artist::new(&entry.artist));
        }
    }
    Err(NotFoundError::Artist)
}

/// Searches the entries for if the given album exists in the dataset
///
/// Wrapped by [`crate::types::Find::album()`]
///
/// # Errors
///
/// This function will return an [`Err`] with [`NotFoundError::Album`]
/// if it cannot find an album with the given name and artist
pub fn find_album(
    entries: &[SongEntry],
    album_name: &str,
    artist_name: &str,
) -> Result<Album, NotFoundError> {
    // .to_lowercase() so that user input capitalization doesn't matter
    // -> problem with different versions of the same album having different
    // capitalization
    // see #2 https://github.com/fsktom/rusty-endsong-parser/issues/2
    let usr_album = Album::new(album_name.to_lowercase(), artist_name.to_lowercase());

    for entry in entries {
        if Album::new(entry.album.to_lowercase(), entry.artist.to_lowercase()).eq(&usr_album) {
            // but here so that the version with proper
            // capitalization is returned
            return Ok(Album::new(&entry.album, &entry.artist));
        }
    }
    Err(NotFoundError::Album)
}

/// Searches the entries for if the given song (in that specific album)
/// exists in the dataset
pub fn find_song_from_album(
    entries: &[SongEntry],
    song_name: &str,
    album_name: &str,
    artist_name: &str,
) -> Result<Song, NotFoundError> {
    // .to_lowercase() so that user input capitalization doesn't matter
    // -> problem with different versions of the same album having different
    // capitalization
    // see #2 https://github.com/fsktom/rusty-endsong-parser/issues/2
    let usr_song = Song::new(
        song_name.to_lowercase(),
        album_name.to_lowercase(),
        artist_name.to_lowercase(),
    );

    for entry in entries {
        if Song::new(
            entry.track.to_lowercase(),
            entry.album.to_lowercase(),
            entry.artist.to_lowercase(),
        )
        .eq(&usr_song)
        {
            // but here so that the version with proper
            // capitalization is returned
            return Ok(Song::new(&entry.track, &entry.album, &entry.artist));
        }
    }
    Err(NotFoundError::Song)
}

/// Searches the dataset for multiple versions of a song
///
/// Returns a [`Vec<Song>`] containing an instance
/// of [`Song`] for every album it's been found in
pub fn find_song(
    entries: &[SongEntry],
    song_name: &str,
    artist_name: &str,
) -> Result<Vec<Song>, NotFoundError> {
    let usr_song = SongJustArtist::new(song_name, artist_name);

    let mut song_versions: Vec<Song> = Vec::new();

    for entry in entries {
        // .to_lowercase() so that user input capitalization doesn't matter
        if entry.track.to_lowercase().eq(&usr_song.name.to_lowercase())
            && entry
                .artist
                .to_lowercase()
                .eq(&usr_song.artist.name.to_lowercase())
        {
            let song_v = Song::new(&entry.track, &entry.album, &entry.artist);
            if !song_versions.contains(&song_v) {
                song_versions.push(song_v);
            }
        }
    }

    if !song_versions.is_empty() {
        return Ok(song_versions);
    }

    Err(NotFoundError::JustSong)
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
        assert!(entries.find().artist("Powerwolf").is_err());
    }
}
