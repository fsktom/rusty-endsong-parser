use std::collections::HashMap;

use crate::types::Aspect;
use crate::types::AspectFull;
use crate::types::Music;
use crate::types::SongEntry;
use crate::types::{Album, Artist, Song};

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

pub fn print_top_from_album(entries: &Vec<SongEntry>, asp: Aspect, album: &Album, num: usize) {
    match asp {
        Aspect::Songs => {
            println!("=== TOP {} SONGS FROM {} ===", num, album);
            print_top_helper(gather_songs_with_album(entries, album), num);
            println!();
        }
        _ => println!("gay"),
    }
}

fn print_top_helper<T: Music>(music_dict: HashMap<T, u32>, num: usize) {
    // https://stackoverflow.com/q/34555837/6694963
    let mut music_vec: Vec<(&T, &u32)> = music_dict.iter().collect();
    music_vec.sort_by(|a, b| b.1.cmp(a.1));
    // TODO: secondary sorting
    //       if plays are equal -> sort A->Z

    // if the number of unique songs/... is lower than the parsed num
    let ind: usize;
    if music_vec.len() < num {
        ind = music_vec.len();
    } else {
        ind = num;
    }

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

fn gather_songs(entries: &Vec<SongEntry>) -> HashMap<Song, u32> {
    let mut songs: HashMap<Song, u32> = HashMap::new();

    for entry in entries {
        let artist = Artist {
            name: entry.artist.clone(),
        };
        let album = Album {
            name: entry.album.clone(),
            artist: artist.clone(),
        };
        let song = Song {
            name: entry.track.clone(),
            album: album.clone(),
            // id: entry.id.clone(),
        };

        // either create new field with value 0 (and add 1 to it)
        // or if a field with that key already exists,
        // add one play to it
        *songs.entry(song).or_insert(0) += 1;
    }

    songs
}

fn gather_songs_with_artist(entries: &Vec<SongEntry>, art: &Artist) -> HashMap<Song, u32> {
    let mut songs: HashMap<Song, u32> = HashMap::new();

    for entry in entries {
        let artist = Artist {
            name: entry.artist.clone(),
        };
        if artist != *art {
            continue;
        }
        let album = Album {
            name: entry.album.clone(),
            artist: artist.clone(),
        };
        let song = Song {
            name: entry.track.clone(),
            album: album.clone(),
            // id: entry.id.clone(),
        };

        *songs.entry(song).or_insert(0) += 1;
    }

    songs
}

fn gather_songs_with_album(entries: &Vec<SongEntry>, alb: &Album) -> HashMap<Song, u32> {
    let mut songs: HashMap<Song, u32> = HashMap::new();

    for entry in entries {
        let artist = Artist {
            name: entry.artist.clone(),
        };
        let album = Album {
            name: entry.album.clone(),
            artist: artist.clone(),
        };
        if album != *alb {
            continue;
        }
        let song = Song {
            name: entry.track.clone(),
            album: album.clone(),
            // id: entry.id.clone(),
        };

        *songs.entry(song).or_insert(0) += 1;
    }

    songs
}

fn gather_albums(entries: &Vec<SongEntry>) -> HashMap<Album, u32> {
    let mut albums: HashMap<Album, u32> = HashMap::new();

    for entry in entries {
        let artist = Artist {
            name: entry.artist.clone(),
        };
        let album = Album {
            name: entry.album.clone(),
            artist: artist.clone(),
        };

        *albums.entry(album).or_insert(0) += 1;
    }

    albums
}

fn gather_albums_with_artist(entries: &Vec<SongEntry>, art: &Artist) -> HashMap<Album, u32> {
    let mut albums: HashMap<Album, u32> = HashMap::new();

    for entry in entries {
        let artist = Artist {
            name: entry.artist.clone(),
        };
        if artist != *art {
            continue;
        }
        let album = Album {
            name: entry.album.clone(),
            artist: artist.clone(),
        };

        *albums.entry(album).or_insert(0) += 1;
    }

    albums
}

fn gather_artists(entries: &Vec<SongEntry>) -> HashMap<Artist, u32> {
    let mut artists: HashMap<Artist, u32> = HashMap::new();

    for entry in entries {
        let artist = Artist {
            name: entry.artist.clone(),
        };

        *artists.entry(artist).or_insert(0) += 1;
    }

    artists
}

struct ArtistPlays(Artist, u32);
struct AlbumPlays(Album, u32);
struct SongPlays(Song, u32);

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

fn gather_artist(entries: &Vec<SongEntry>, art: &Artist) -> ArtistPlays {
    let mut artist_asp = ArtistPlays(art.clone(), 0);

    for entry in entries {
        let artist = Artist {
            name: entry.artist.clone(),
        };

        if artist == *art {
            artist_asp.1 += 1;
        }
    }

    artist_asp
}

fn gather_album(entries: &Vec<SongEntry>, alb: &Album) -> AlbumPlays {
    let mut album_asp = AlbumPlays(alb.clone(), 0);

    for entry in entries {
        let artist = Artist {
            name: entry.artist.clone(),
        };
        let album = Album {
            name: entry.album.clone(),
            artist: artist.clone(),
        };

        if album == *alb {
            album_asp.1 += 1;
        }
    }

    album_asp
}

fn gather_song(entries: &Vec<SongEntry>, son: &Song) -> SongPlays {
    let mut song_asp = SongPlays(son.clone(), 0);

    for entry in entries {
        let artist = Artist {
            name: entry.artist.clone(),
        };
        let album = Album {
            name: entry.album.clone(),
            artist: artist.clone(),
        };
        let song = Song {
            name: entry.track.clone(),
            album: album.clone(),
            // id: entry.id.clone(),
        };

        if song == *son {
            song_asp.1 += 1;
        }
    }

    song_asp
}

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
