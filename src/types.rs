// https://doc.rust-lang.org/stable/book/ch06-01-defining-an-enum.html

pub fn run() {
    let art1 = Artist {
        name: String::from("Powerwolf"),
    };
    let alb1 = Album {
        name: String::from("Bible of the Beast"),
        artist: art1,
    };
    // let son1 = Song {
    //     name: String::from("Midnight Messiah"),
    //     album: alb1,
    //     artist: art1,
    // };

    let som = Aspect::Album(alb1);
}

pub enum Aspect {
    Artist(Artist),
    Album(Album),
    Song(Song),
}

pub struct Artist {
    name: String,
}

pub struct Album {
    name: String,
    artist: Artist,
}

pub struct Song {
    name: String,
    album: Album,
    artist: Artist,
}
