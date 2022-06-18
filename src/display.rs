use std::collections::HashMap;

use crate::types::Aspect;
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
            println!("=== TOP {} SONGS FROM {} ===", num, artist.name);
            print_top_helper(gather_songs_with_artist(entries, artist), num);
            println!();
        }
        Aspect::Albums => {
            println!("=== TOP {} ALBUMS FROM {} ===", num, artist.name);
            print_top_helper(gather_albums_with_artist(entries, artist), num);
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
        println!("#{}\t{} => {}", i + 1, m, n)
    }
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
            id: entry.id.clone(),
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
            id: entry.id.clone(),
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
