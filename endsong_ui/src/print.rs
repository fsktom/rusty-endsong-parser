//! Module responsible for displaying the contents of endsong.json files
//! in a human-readable format (e.g. as 100 most played songs)
//! to the [`std::io::stdout`]

use std::cmp::Reverse;
use std::collections::HashMap;
use std::fmt::Display;
use std::str::FromStr;

use itertools::Itertools;

use endsong::prelude::*;

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
impl FromStr for Aspect {
    // temporary, if tryerror addded adapt this
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "artist" | "artists" => Ok(Aspect::Artists),
            "album" | "albums" => Ok(Aspect::Albums),
            "song" | "songs" => Ok(Aspect::Songs),
            _ => Err(()),
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
/// Used in [`top_from_artist()`]
#[derive(Copy, Clone, Debug)]
pub enum Mode {
    /// to print albums from artist
    Albums,
    /// to print songs from artists
    Songs,
}

/// Trait for better display of [`Durations`][Duration]
pub trait DurationUtils {
    /// Returns a string with the duration in the format `HH:MM:SS`
    /// or `MM:SS` (if the duration is less than an hour)
    fn display(&self) -> String;
}
impl DurationUtils for Duration {
    fn display(&self) -> String {
        let hours = self.num_hours();
        let seconds = self.num_seconds() % 60;
        if hours > 0 {
            let minutes = self.num_minutes() % hours;
            format!("{hours:02}:{minutes:02}:{seconds:02}")
        } else {
            let minutes = self.num_minutes();
            format!("{minutes:02}:{seconds:02}")
        }
    }
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
        }
        Aspect::Albums => {
            println!("=== TOP {num} ALBUMS ===");
            top_helper(gather::albums(entries), num);
        }
        Aspect::Artists => {
            println!("=== TOP {num} ARTISTS ===");
            top_helper(gather::artists(entries), num);
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
        }
        Mode::Albums => {
            println!("=== TOP {num} ALBUMS FROM {artist} ===");
            top_helper(gather::albums_from_artist(entries, artist), num);
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
}

/// Used by [`top()`]
fn top_helper<Asp: Music>(music_dict: HashMap<Asp, usize>, num: usize) {
    let music_vec: Vec<(Asp, usize)> = music_dict
        .into_iter()
        // primary sorting: by plays descending
        // https://stackoverflow.com/a/34555984
        // https://stackoverflow.com/a/60916195
        // and secondary sorting by name ascending
        .sorted_unstable_by_key(|t| (Reverse(t.1), t.0.clone()))
        .collect_vec();
    let length = music_vec.len();

    // if the number of unique aspects is lower than the parsed num
    let max_num: usize = if length < num { length } else { num };

    for (i, (asp, plays)) in music_vec.iter().enumerate() {
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
            println!("{} | {} plays", art, gather::plays(entries, art));
            artist(entries, &gather::albums_from_artist(entries, art));
        }
        AspectFull::Album(alb) => {
            println!("{} | {} plays", alb, gather::plays(entries, alb));
            album(&gather::songs_from(entries, alb), 4);
        }
        AspectFull::Song(son) => {
            println!("{} | {} plays", son, gather::plays(entries, son));
        }
    }
}

/// Prints each [`Album`] of `albums` with the playcount
///
/// Preferably `albums` contains only albums from one artist
fn artist(entries: &[SongEntry], albums: &HashMap<Album, usize>) {
    // albums sorted by their playcount descending (primary)
    // and name ascending (secondary) if plays are equal
    let albums_vec: Vec<(&Album, &usize)> = albums
        .iter()
        .sorted_unstable_by_key(|t| (Reverse(t.1), t.0))
        .collect_vec();

    for (alb, plays) in albums_vec {
        println!("    {} | {plays} plays", alb.name);
        album(&gather::songs_from(entries, alb), 8);
    }
}

/// Prints each [`Song`] of `songs` with the playcount
///
/// Preferably `songs` contains only songs from one album
fn album(songs: &HashMap<Song, usize>, indent: usize) {
    // songs sorted by their playcount descending (primary)
    // and name ascending (secondary) if plays are equal
    let songs_vec: Vec<(&Song, &usize)> = songs
        .iter()
        .sorted_unstable_by_key(|t| (Reverse(t.1), t.0))
        .collect_vec();

    let indent = " ".repeat(indent);

    for (song, plays) in songs_vec {
        println!("{indent}{} | {plays} plays", song.name);
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
    entries: &SongEntries,
    asp: &AspectFull,
    start: &DateTime<Local>,
    end: &DateTime<Local>,
) {
    assert!(start <= end, "Start date is after end date!");
    let entries_within_dates = entries.between(start, end);

    let (start, end) = normalize_dates(entries_within_dates, start, end);

    match *asp {
        AspectFull::Artist(art) => {
            println!(
                "{} | between {} and {} | {} plays",
                art,
                start.date_naive(),
                end.date_naive(),
                gather::plays(entries_within_dates, art)
            );
            artist(
                entries_within_dates,
                &gather::albums_from_artist(entries_within_dates, art),
            );
        }
        AspectFull::Album(alb) => {
            println!(
                "{} | between {} and {} | {} plays",
                alb,
                start.date_naive(),
                end.date_naive(),
                gather::plays(entries_within_dates, alb)
            );
            album(&gather::songs_from(entries_within_dates, alb), 4);
        }
        AspectFull::Song(son) => {
            println!(
                "{} | between {} and {} | {} plays",
                son,
                start.date_naive(),
                end.date_naive(),
                gather::plays(entries_within_dates, son)
            );
        }
    }
}

/// Prints the total time played
#[allow(clippy::missing_panics_doc)]
pub fn time_played(entries: &SongEntries) {
    time_played_date(
        entries,
        &entries.first().unwrap().timestamp,
        &entries.last().unwrap().timestamp,
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
pub fn time_played_date(entries: &SongEntries, start: &DateTime<Local>, end: &DateTime<Local>) {
    assert!(start <= end, "Start date is after end date!");
    let duration = gather::listening_time(entries.between(start, end));
    let (start, end) = normalize_dates(entries, start, end);
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
        gather::all_plays(entries.between(start, end)) as i64 / period.num_days(),
        duration.num_hours() / period.num_days(),
    );
}

/// Used by `*_date` functions to set the start date to
/// the first entry's date and the end date to the last entry's date
/// if the inputted dates are before/after those dates
fn normalize_dates<'a>(
    entries: &'a [SongEntry],
    start: &'a DateTime<Local>,
    end: &'a DateTime<Local>,
) -> (&'a DateTime<Local>, &'a DateTime<Local>) {
    // if inputted start date is before the actual first entry
    // it should be changed to the first entry's date
    let first = entries.first().unwrap();
    let start = if &first.timestamp > start {
        &first.timestamp
    } else {
        start
    };

    // if inputted end date is after the actual last entry
    // it should be changed to the last entry's date
    let last = entries.last().unwrap();
    let end = if &last.timestamp < end {
        &last.timestamp
    } else {
        end
    };

    (start, end)
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
/// ```ignore
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
