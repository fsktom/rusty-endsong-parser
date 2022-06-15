// https://doc.rust-lang.org/stable/book/ch06-01-defining-an-enum.html
use std::fmt::Display;

pub fn run() {
    let art1 = Artist {
        name: String::from("Powerwolf"),
    };
    let alb1 = Album {
        name: String::from("Bible of the Beast"),
        artist: &art1,
    };
    let son1 = Song {
        name: String::from("Midnight Messiah"),
        album: &alb1,
        artist: &art1,
    };

    let som_art = Aspect::Artist(&art1);
    let som_alb = Aspect::Album(&alb1);
    let som_son = Aspect::Song(&son1);
    println!("{}", som_art);
    println!("{}", som_alb);
    println!("{}", som_son);
}

pub enum Aspect<'a> {
    Artist(&'a Artist),
    Album(&'a Album<'a>),
    Song(&'a Song<'a>),
}

impl Display for Aspect<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            &Aspect::Artist(art) => write!(f, "{}", art.name),
            &Aspect::Album(alb) => write!(f, "{} - {}", alb.artist.name, alb.name),
            &Aspect::Song(son) => {
                write!(f, "{} - {} ({})", son.artist.name, son.name, son.album.name)
            }
        }
    }
}

pub struct Artist {
    name: String,
}

pub struct Album<'a> {
    name: String,
    artist: &'a Artist,
}

pub struct Song<'a> {
    name: String,
    album: &'a Album<'a>,
    artist: &'a Artist,
}
