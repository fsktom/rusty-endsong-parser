//! Module responsible for displaying the contents of endsong.json files
//! in a human-readable format (e.g. as 100 most played songs)
//! to the [std::io::stdout]
use std::collections::HashMap;

use crate::types::Aspect;
use crate::types::AspectFull;
use crate::types::Music;
use crate::types::SongEntry;
use crate::types::{Album, Artist, Song};

/// If set to true, it will sum up the plays of one song across multiple
/// albums it may be in
///
/// Only applies to printing top songs with [print_top()]!
///
/// The album displayed in the parantheses will be the one it has the
/// highest amount of listens from
pub const SUM_ALBUMS: bool = true;

/// Prints the top `num` of an `asp`
///
/// * `asp` - [Aspect::Songs] (affected by [SUM_ALBUMS]) for top songs, [Aspect::Albums] for top albums and
/// [Aspect::Artists] for top artists
/// * `num` - number of displayed top aspects. Will automatically change to total number of that aspect if `num` is higher than that
pub fn print_top(entries: &Vec<SongEntry>, asp: Aspect, num: usize) {
    match asp {
        Aspect::Songs => {
            println!("=== TOP {} SONGS ===", num);
            print_top_helper(gather_songs(entries), num);
            println!();
        }
        Aspect::Albums => {
            println!("=== TOP {} ALBUMS ===", num);
            print_top_helper(gather_albums(entries), num);
            println!();
        }
        Aspect::Artists => {
            println!("=== TOP {} ARTISTS ===", num);
            print_top_helper(gather_artists(entries), num);
            println!();
        }
    }
}

/// Prints top songs or albums from an artist
///
/// * `asp` - [Aspect::Songs] for top songs and [Aspect::Albums] for top albums
/// * `artist` - the [Artist] you want the top songs/albums from
/// * `num` - number of displayed top aspects. Will automatically change to total number of that aspect if `num` is higher than that
pub fn print_top_from_artist(entries: &Vec<SongEntry>, asp: Aspect, artist: &Artist, num: usize) {
    match asp {
        Aspect::Songs => {
            println!("=== TOP {} SONGS FROM {} ===", num, artist);
            print_top_helper(gather_songs_with_artist(entries, artist), num);
            println!();
        }
        Aspect::Albums => {
            println!("=== TOP {} ALBUMS FROM {} ===", num, artist);
            print_top_helper(gather_albums_with_artist(entries, artist), num);
            println!();
        }
        _ => println!("gay"),
    }
}

/// Prints top songs from an album
///
/// * `album` - the [Album] you want the top songs from
/// * `num` - number of displayed top songs.
/// Will automatically change to total number of songs from that album if `num` is higher than that
pub fn print_top_from_album(entries: &Vec<SongEntry>, album: &Album, num: usize) {
    println!("=== TOP {} SONGS FROM {} ===", num, album);
    print_top_helper(gather_songs_with_album(entries, album), num);
    println!();
}

/// Used by [print_top()]
fn print_top_helper<T: Music>(music_dict: HashMap<T, u32>, num: usize) {
    // https://stackoverflow.com/q/34555837/6694963
    let mut music_vec: Vec<(&T, &u32)> = music_dict.iter().collect();
    music_vec.sort_by(|a, b| b.1.cmp(a.1));
    // TODO: secondary sorting
    //       if plays are equal -> sort A->Z

    // if the number of unique songs/... is lower than the parsed num
    let ind: usize = if music_vec.len() < num {
        music_vec.len()
    } else {
        num
    };

    for i in 0..ind {
        let mus = music_vec.get(i).unwrap();
        let m = mus.0;
        let n = mus.1;
        println!("{}: {} | {} plays", leading_whitespace(i + 1, ind), m, n)
    }
}

/// Formats `1` to ` #1` if user wishes for Top 10
/// or to `  #1` if Top 100 etc.
/// # Examples
/// ```
/// assert_eq!(leading_whitespace(7usize, 100usize), String::from("  #7"));
/// assert_eq!(leading_whitespace(7usize, 1000usize), String::from("   #7"));
/// ```
fn leading_whitespace(num: usize, max_num: usize) -> String {
    // https://github.com/Filip-Tomasko/endsong-parser-python/blob/main/src/endsong_parser.py#L551-L578
    let mut order_format = String::new();

    // bc as of Rust 1.62 it doesn't support log10 on usize
    // https://doc.rust-lang.org/std/primitive.usize.html#method.log10
    let num = num as f64;
    let max_num = max_num as f64;

    let mut num_of_zero = max_num.log10().floor() as usize;
    let digits = num.log10() as usize + 1;

    loop {
        if num_of_zero == 0 {
            break;
        }
        if digits <= num_of_zero {
            order_format += " ";
        }
        num_of_zero -= 1;
    }

    format!("{}#{}", order_format, num)
}

/// basically [Song] but without the [Album] field
/// used in [print_top] if [SUM_ALBUMS] is set to true
/// and in [find_song]
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
struct SongJustArtist {
    /// Name of the song
    name: String,
    /// Artist of the song
    artist: Artist,
}

/// Used by [print_top_helper()]
#[allow(clippy::needless_range_loop)]
fn gather_songs(entries: &Vec<SongEntry>) -> HashMap<Song, u32> {
    let mut songs: HashMap<Song, u32> = HashMap::new();

    for entry in entries {
        let song = Song::new(
            entry.track.clone(),
            entry.album.clone(),
            entry.artist.clone(),
        );

        // either create new field with value 0 (and add 1 to it)
        // or if a field with that key already exists,
        // add one play to it
        *songs.entry(song).or_insert(0) += 1;
    }

    if SUM_ALBUMS {
        let mut songs_artist: HashMap<SongJustArtist, u32> = HashMap::new();

        /// tuple struct containing an Album with the amount of plays
        #[derive(PartialEq, Eq, Hash, Debug, Clone)]
        struct AlbumPlays(Album, u32);

        /// contains the name of the song and
        /// a vector containg all the albums this song is in
        #[derive(PartialEq, Eq, Hash, Debug, Clone)]
        struct SongAlbums {
            /// Name of the song
            name: String,
            /// Vector with the albums the song is in with
            /// the amount of plays in each album
            albums: Vec<AlbumPlays>,
        }
        // to know which album the song had highest amount of plays from
        // that album will be then displayed in () after the song name
        // but the number of plays that will be displayed will be a sum of
        // the plays from all albums
        let mut changed: HashMap<SongJustArtist, SongAlbums> = HashMap::new();

        for (k, v) in songs.iter() {
            let song_just_artist = SongJustArtist {
                name: k.name.clone(),
                artist: k.album.artist.clone(),
            };

            match songs_artist.get(&song_just_artist) {
                // if it finds something it means that the song
                // only from a different album already exists
                Some(plays) => {
                    *songs_artist.entry(song_just_artist.clone()).or_insert(0) += *plays;

                    let temp = changed.get_mut(&song_just_artist).unwrap();
                    temp.albums.push(AlbumPlays(k.album.clone(), *v));
                }
                // if it doesn't find anything, it's the first appearance of that song
                None => {
                    songs_artist.insert(song_just_artist.clone(), *v);
                    let salb = SongAlbums {
                        name: k.name.clone(),
                        albums: vec![AlbumPlays(k.album.clone(), *v)],
                    };
                    changed.insert(song_just_artist.clone(), salb);
                }
            }
        }

        for (k, v) in changed.iter() {
            let albs = &v.albums;

            // the first album will be taken if both have
            // the same number of plays
            let mut total: u32 = 0;
            let highest: &AlbumPlays = {
                let mut plays = 0;
                for alb in 0..albs.len() {
                    if albs[alb].1 > plays {
                        plays = albs[alb].1
                    }
                    total += albs[alb].1
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

/// Used by [print_top_helper()]
fn gather_songs_with_artist(entries: &Vec<SongEntry>, art: &Artist) -> HashMap<Song, u32> {
    let mut songs: HashMap<Song, u32> = HashMap::new();

    for entry in entries {
        if Artist::new(entry.artist.clone()) != *art {
            continue;
        }

        let song = Song::new(
            entry.track.clone(),
            entry.album.clone(),
            entry.artist.clone(),
        );

        *songs.entry(song).or_insert(0) += 1;
    }

    songs
}

/// Used by [print_top_helper()] and [print_album()]
fn gather_songs_with_album(entries: &Vec<SongEntry>, alb: &Album) -> HashMap<Song, u32> {
    let mut songs: HashMap<Song, u32> = HashMap::new();

    for entry in entries {
        if Album::new(entry.album.clone(), entry.artist.clone()) != *alb {
            continue;
        }

        let song = Song::new(
            entry.track.clone(),
            entry.album.clone(),
            entry.artist.clone(),
        );

        *songs.entry(song).or_insert(0) += 1;
    }

    songs
}

/// Used by [print_top_helper()]
fn gather_albums(entries: &Vec<SongEntry>) -> HashMap<Album, u32> {
    let mut albums: HashMap<Album, u32> = HashMap::new();

    for entry in entries {
        let album = Album::new(entry.album.clone(), entry.artist.clone());

        *albums.entry(album).or_insert(0) += 1;
    }

    albums
}

/// Used by [print_top_helper()]
fn gather_albums_with_artist(entries: &Vec<SongEntry>, art: &Artist) -> HashMap<Album, u32> {
    let mut albums: HashMap<Album, u32> = HashMap::new();

    for entry in entries {
        if Artist::new(entry.artist.clone()) != *art {
            continue;
        }
        let album = Album::new(entry.album.clone(), entry.artist.clone());
        *albums.entry(album).or_insert(0) += 1;
    }

    albums
}

/// Used by [print_top_helper()]
fn gather_artists(entries: &Vec<SongEntry>) -> HashMap<Artist, u32> {
    let mut artists: HashMap<Artist, u32> = HashMap::new();

    for entry in entries {
        let artist = Artist::new(entry.artist.clone());

        *artists.entry(artist).or_insert(0) += 1;
    }

    artists
}

/// Tuple struct containing the artist with the amount of plays
struct ArtistPlays(Artist, u32);
/// Tuple struct containing the album with the amount of plays
struct AlbumPlays(Album, u32);
/// Tuple struct containing the song with the amount of plays
struct SongPlays(Song, u32);

/// Prints a specfic aspect
///
/// * `asp` - the aspect you want informationa about containing the
/// relevant struct
pub fn print_aspect(entries: &Vec<SongEntry>, asp: AspectFull) {
    match asp {
        AspectFull::Artist(art) => {
            println!("=== {} | {} plays ===", art, gather_artist(entries, art).1);
            print_artist(entries, gather_albums_with_artist(entries, art));
        }
        AspectFull::Album(alb) => {
            println!("=== {} | {} plays ===", alb, gather_album(entries, alb).1);
            print_album(gather_songs_with_album(entries, alb));
        }
        AspectFull::Song(son) => {
            let son = gather_song(entries, son);
            println!("{} | {} plays", son.0, son.1);
        }
    }
}

/// Counts up the plays of a single artist
fn gather_artist(entries: &Vec<SongEntry>, art: &Artist) -> ArtistPlays {
    let mut artist_asp = ArtistPlays(art.clone(), 0);

    for entry in entries {
        let artist = Artist::new(entry.artist.clone());

        if artist == *art {
            artist_asp.1 += 1;
        }
    }

    artist_asp
}

/// Counts up the plays of a single album
fn gather_album(entries: &Vec<SongEntry>, alb: &Album) -> AlbumPlays {
    let mut album_asp = AlbumPlays(alb.clone(), 0);

    for entry in entries {
        let album = Album::new(entry.album.clone(), entry.artist.clone());

        if album == *alb {
            album_asp.1 += 1;
        }
    }

    album_asp
}

/// Counts up the plays of a single song
fn gather_song(entries: &Vec<SongEntry>, son: &Song) -> SongPlays {
    let mut song_asp = SongPlays(son.clone(), 0);

    for entry in entries {
        let song = Song::new(
            entry.track.clone(),
            entry.album.clone(),
            entry.artist.clone(),
        );

        if song == *son {
            song_asp.1 += 1;
        }
    }

    song_asp
}

/// Used by [print_aspect()]
fn print_artist(entries: &Vec<SongEntry>, artist: HashMap<Album, u32>) {
    let mut artist_vec: Vec<(&Album, &u32)> = artist.iter().collect();
    artist_vec.sort_by(|a, b| b.1.cmp(a.1));

    for i in 0..artist_vec.len() {
        let alb = artist_vec.get(i).unwrap().0;
        let mus = gather_songs_with_album(entries, alb);
        // calling gather_album here is unnecessary work
        // it should add up the total plays somehwere else
        println!("--- {} | {} plays ---", alb, gather_album(entries, alb).1);
        print_album(mus);
    }
}

/// Used by [print_aspect()]
fn print_album(album: HashMap<Song, u32>) {
    let mut album_vec: Vec<(&Song, &u32)> = album.iter().collect();
    album_vec.sort_by(|a, b| b.1.cmp(a.1));

    for i in 0..album_vec.len() {
        let mus = album_vec.get(i).unwrap();
        let m = mus.0;
        let n = mus.1;
        println!(
            "{}: {} | {} plays",
            leading_whitespace(i + 1, album_vec.len()),
            m,
            n
        )
    }
}

/// Searches the entries for if the given artist exists in the dataset
pub fn find_artist(entries: &Vec<SongEntry>, artist_name: String) -> Option<Artist> {
    let usr_artist = Artist::new(artist_name);

    for entry in entries {
        // .to_lowercase() so that user input capitalization doesn't matter
        if entry
            .artist
            .to_lowercase()
            .eq(&usr_artist.name.to_lowercase())
        {
            return Some(usr_artist);
        }
    }
    None
}

/// Searches the entries for if the given album exists in the dataset
pub fn find_album(
    entries: &Vec<SongEntry>,
    album_name: String,
    artist_name: String,
) -> Option<Album> {
    let usr_album = Album::new(album_name, artist_name);

    for entry in entries {
        // .to_lowercase() so that user input capitalization doesn't matter
        if entry.album.to_lowercase().eq(&usr_album.name)
            && entry
                .artist
                .to_lowercase()
                .eq(&usr_album.artist.name.to_lowercase())
        {
            return Some(usr_album);
        }
    }
    None
}

/// Searches the entries for if the given song (in that specific album)
/// exists in the dataset
pub fn find_song_from_album(
    entries: &Vec<SongEntry>,
    song_name: String,
    album_name: String,
    artist_name: String,
) -> Option<Song> {
    // .to_lowercase() so that user input capitalization doesn't matter
    let usr_song = Song::new(
        song_name.to_lowercase(),
        album_name.to_lowercase(),
        artist_name.to_lowercase(),
    );

    for entry in entries {
        if Song::new(
            entry.track.clone().to_lowercase(),
            entry.album.clone().to_lowercase(),
            entry.artist.clone().to_lowercase(),
        )
        .eq(&usr_song)
        {
            // but here so that the version with proper
            // capitalization is returned
            return Some(Song::new(
                entry.track.clone(),
                entry.album.clone(),
                entry.artist.clone(),
            ));
        }
    }
    None
}

/// Searches the dataset for multiple versions of a song
///
/// Returns a [Vec<Song>] containing an instance
/// of [Song] for every album it's been found in
pub fn find_song(
    entries: &Vec<SongEntry>,
    song_name: String,
    artist_name: String,
) -> Option<Vec<Song>> {
    let usr_song = SongJustArtist {
        name: song_name,
        artist: Artist::new(artist_name),
    };

    let mut song_versions: Vec<Song> = Vec::new();

    for entry in entries {
        // .to_lowercase() so that user input capitalization doesn't matter
        if entry.track.to_lowercase().eq(&usr_song.name.to_lowercase())
            && entry
                .artist
                .to_lowercase()
                .eq(&usr_song.artist.name.to_lowercase())
        {
            let song_v = Song::new(
                entry.track.clone(),
                entry.album.clone(),
                entry.artist.clone(),
            );
            if !song_versions.contains(&song_v) {
                song_versions.push(song_v);
            }
        }
    }

    println!("{:?}", song_versions);

    if !song_versions.is_empty() {
        return Some(song_versions);
    }

    None
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
}
