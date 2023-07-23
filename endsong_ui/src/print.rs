//! Module responsible for displaying the contents of endsong.json files
//! in a human-readable format (e.g. as 100 most played songs)
//! to the [`std::io::stdout`]

use std::collections::HashMap;
use std::fmt::Display;

use chrono::DateTime;
use chrono_tz::Tz;

use crate::gather;
use crate::types::Music;
use crate::types::SongEntry;
use crate::types::{Album, Artist, Song};

/// An enum that is among other things used by functions such as
/// [`top()`] and its derivatives to know whether
/// to print top songs ([`Aspect::Songs`]), albums ([`Aspect::Albums`])
/// or artists ([`Aspect::Artists`])
#[derive(Copy, Clone, Debug, Default)]
pub enum Aspect {
    /// to print top artists
    Artists,
    /// to print top albums
    Albums,
    // bc Rust still doesn't have default argument values
    // https://www.reddit.com/r/rust/comments/fi6nov/why_does_rust_not_support_default_arguments/fkfezxv/
    /// to print top songs
    #[default]
    Songs,
}
impl Display for Aspect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Aspect::Artists => write!(f, "artists"),
            Aspect::Albums => write!(f, "albums"),
            Aspect::Songs => write!(f, "songs"),
        }
    }
}

/// Algebraic data type similar to [`Aspect`]
/// but used by functions such as [`crate::print::aspect()`]
/// to get more specfic data
///
/// Each variant contains a reference to an instance of the aspect
pub enum AspectFull<'a> {
    /// with ref to [`Artist`]
    Artist(&'a Artist),
    /// with ref to [`Album`]
    Album(&'a Album),
    /// with ref to [`Song`]
    Song(&'a Song),
}

/// For choosing mode of a function, similar to [`Aspect`] but
/// without [`Aspect::Artists`]
///
/// Used in [`crate::print::top_from_artist()`]
#[derive(Copy, Clone, Debug)]
pub enum Mode {
    /// to print albums from artist
    Albums,
    /// to print songs from artists
    Songs,
}

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
pub fn top(entries: &[SongEntry], asp: Aspect, num: usize, sum_songs_from_different_albums: bool) {
    match asp {
        Aspect::Songs => {
            println!("=== TOP {num} SONGS ===");
            top_helper(gather::songs(entries, sum_songs_from_different_albums), num);
            println!();
        }
        Aspect::Albums => {
            println!("=== TOP {num} ALBUMS ===");
            top_helper(gather::albums(entries), num);
            println!();
        }
        Aspect::Artists => {
            println!("=== TOP {num} ARTISTS ===");
            top_helper(gather::artists(entries), num);
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
pub fn top_from_artist(entries: &[SongEntry], mode: Mode, artist: &Artist, num: usize) {
    match mode {
        Mode::Songs => {
            println!("=== TOP {num} SONGS FROM {artist} ===");
            top_helper(gather::songs_from(entries, artist), num);
            println!();
        }
        Mode::Albums => {
            println!("=== TOP {num} ALBUMS FROM {artist} ===");
            top_helper(gather::albums_from_artist(entries, artist), num);
            println!();
        }
    }
}

/// Prints top songs from an album
///
/// * `album` - the [`Album`] you want the top songs from
/// * `num` - number of displayed top songs.
/// Will automatically change to total number of songs from that album if `num` is higher than that
pub fn top_from_album(entries: &[SongEntry], album: &Album, num: usize) {
    println!("=== TOP {num} SONGS FROM {album} ===");
    top_helper(gather::songs_from(entries, album), num);
    println!();
}

/// Used by [`print_top()`]
fn top_helper<Asp: Music>(music_dict: HashMap<Asp, usize>, num: usize) {
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

/// Prints a specfic aspect
///
/// * `asp` - the [`AspectFull`] you want information about containing the
/// relevant struct ([`Artist`], [`Album`] or [`Song`])
pub fn aspect(entries: &[SongEntry], asp: &AspectFull) {
    match *asp {
        AspectFull::Artist(art) => {
            println!("=== {} | {} plays ===", art, gather::plays(entries, art));
            // TODO! currently print_artist uses the whole time for num of plays!!!
            // e.g. printing Alestorm between 2022-01-01 and 2022-07-01
            // on only `endsong_0.json`
            // will print:
            // === Alestorm between 2022-01-01CET and 2022-07-01CEST | 1 plays ===
            // --- Alestorm - Sunset On The Golden Age | 3 plays ---
            // #1: Alestorm - Drink (Sunset On The Golden Age) | 3 plays

            artist(entries, &gather::albums_from_artist(entries, art));
        }
        AspectFull::Album(alb) => {
            println!("=== {} | {} plays ===", alb, gather::plays(entries, alb));
            // TODO! currently print_album uses the whole time for num of plays!!!
            album(&gather::songs_from(entries, alb));
        }
        AspectFull::Song(son) => {
            println!("{} | {} plays", son, gather::plays(entries, son));
        }
    }
}

/// Prints each [`Album`] of `albums` with the playcount
fn artist(entries: &[SongEntry], albums: &HashMap<Album, usize>) {
    let mut albums_vec: Vec<(&Album, &usize)> = albums.iter().collect();
    albums_vec.sort_by(|a, b| b.1.cmp(a.1));

    for (alb, plays) in albums_vec {
        println!("--- {alb} | {plays} plays ---");
        album(&gather::songs_from(entries, alb));
    }
}

/// Prints each [`Song`] of `songs` with the playcount
fn album(songs: &HashMap<Song, usize>) {
    let mut songs_vec: Vec<(&Song, &usize)> = songs.iter().collect();
    songs_vec.sort_by(|a, b| b.1.cmp(a.1));

    for (i, (song, plays)) in songs_vec.iter().enumerate() {
        println!(
            "{}: {song} | {plays} plays",
            leading_whitespace(i + 1, songs_vec.len())
        );
    }
}

/// Prints a specfic aspect in a date range
///
/// Basically [`aspect()`] but with date limitations
///
/// * `asp` - the [`AspectFull`] you want information about containing the
/// relevant struct ([`Artist`], [`Album`] or [`Song`])
///
/// # Panics
///
/// Panics if `start` is after or equal to `end`
pub fn aspect_date(
    entries: &[SongEntry],
    asp: &AspectFull,
    start: &DateTime<Tz>,
    end: &DateTime<Tz>,
) {
    assert!(start <= end, "Start date is after end date!");
    match *asp {
        AspectFull::Artist(art) => {
            println!(
                "=== {} between {} and {} | {} plays ===",
                art,
                start.date_naive(),
                end.date_naive(),
                gather::plays_date(entries, art, start, end)
            );
            artist_date(
                entries,
                &gather::albums_from_artist_date(entries, art, start, end),
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
                gather::plays_date(entries, alb, start, end)
            );
            album(&gather::songs_from_date(entries, alb, start, end));
        }
        AspectFull::Song(son) => {
            println!(
                "{} between {} and {} | {} plays",
                son,
                start.date_naive(),
                end.date_naive(),
                gather::plays_date(entries, son, start, end)
            );
        }
    }
}

/// Prints each [`Album`] of `albums` with the playcount in the date range
///
/// # Panics
///
/// Panics if `start` is after or equal to `end`
fn artist_date(
    entries: &[SongEntry],
    albums: &HashMap<Album, usize>,
    start: &DateTime<Tz>,
    end: &DateTime<Tz>,
) {
    assert!(start <= end, "Start date is after end date!");
    let mut albums_vec: Vec<(&Album, &usize)> = albums.iter().collect();
    albums_vec.sort_by(|a, b| b.1.cmp(a.1));

    for (alb, plays) in albums_vec {
        println!("--- {alb} | {plays} plays ---");
        album(&gather::songs_from_date(entries, alb, start, end));
    }
}

/// Prints the total time played
pub fn time_played(entries: &[SongEntry]) {
    let duration = gather::listening_time(entries);

    println!(
        "You've spent {} days - or {} hours - or {} minutes listening to music!",
        &duration.num_days(),
        &duration.num_hours(),
        &duration.num_minutes()
    );
}

/// Prints the time played in a date range
///
/// Basically [`time_played()`] but with date limitation
///
/// # Panics
///
/// Panics if `start` is after or equal to `end`
#[allow(clippy::cast_precision_loss)]
pub fn time_played_date(entries: &[SongEntry], start: &DateTime<Tz>, end: &DateTime<Tz>) {
    assert!(start <= end, "Start date is after end date!");
    let duration = gather::listening_time_date(entries, start, end);
    let period = *end - *start;

    println!(
        "You've spent {} days ({:.2}%) ({} hours / {} minutes) listening to music between {} and {} ({} days à {} plays/day & {} hours/day)!",
        &duration.num_days(),
        ((duration.num_minutes() as f64) / (period.num_minutes() as f64)) * 100.0,
        &duration.num_hours(),
        &duration.num_minutes(),
        start.date_naive(),
        end.date_naive(),
        period.num_days(),
        gather::all_plays_date(entries, start, end) as i64 / period.num_days(),
        duration.num_hours() / period.num_days(),
    );
}

/// Formats `1` to ` #1` if user wishes for Top 10
/// or to `  #1` if Top 100 etc.
///
/// # Arguments
/// * `num` - position of the [`AspectFull`], must be >0
/// * `max_num` - the highest position you want to display,
/// must be >0 and >=`num`
///
/// # Panics
///
/// Panics if `num` or `max_num` is 0
///
/// # Examples
/// ```
/// assert_eq!(leading_whitespace(7usize, 100usize), "  #7");
/// assert_eq!(leading_whitespace(7usize, 1000usize), "   #7");
/// ```
fn leading_whitespace(num: usize, max_num: usize) -> String {
    assert!(num > 0);
    assert!(max_num > 0);
    assert!(max_num >= num);
    // https://github.com/Filip-Tomasko/endsong-parser-python/blob/main/src/endsong_parser.py#L551-L578

    let total_width = max_num.ilog10();
    let num_width = num.ilog10();

    let missing_whitespace = total_width - num_width;

    format!("{}#{num}", " ".repeat(missing_whitespace as usize))
}

// https://doc.rust-lang.org/book/ch11-03-test-organization.html#unit-tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn order_format() {
        assert_eq!(leading_whitespace(3usize, 9usize), "#3");
        assert_eq!(leading_whitespace(3usize, 10usize), " #3");
        assert_eq!(leading_whitespace(3usize, 100usize), "  #3");
        assert_eq!(leading_whitespace(3usize, 1000usize), "   #3");
        assert_eq!(leading_whitespace(3usize, 5692usize), "   #3");
    }

    #[test]
    #[should_panic]
    fn order_format_panics() {
        leading_whitespace(0usize, 100usize);
        leading_whitespace(1usize, 0usize);
        leading_whitespace(101usize, 50usize);
    }
}
