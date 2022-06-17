
use std::collections::HashMap;

use crate::types::Aspect;
use crate::types::SongEntry;
use crate::types::{Album, Artist, Song};

pub fn print_top(entries: &Vec<SongEntry>, asp: Aspect, num: usize) {
    match asp {
        Aspect::Songs => {
            let a = gather_songs(entries);
            // println!("{:?}", a);
            // https://stackoverflow.com/q/34555837/6694963
            let mut a_vec: Vec<(&Song, &u32)> = a.iter().collect();
            a_vec.sort_by(|a, b| b.1.cmp(a.1));
            println!("{:?}", a_vec);
            for i in 0..num {
                let son = a_vec.get(i).unwrap();
                let s = son.0;
                let n = son.1;
                println!("#{}\t{} => {}", i + 1, s, n)
            }
        }
        _ => println!("whatcha doin?"),
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
