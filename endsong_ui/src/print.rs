//! Module responsible for displaying the contents of endsong.json files
//! in a human-readable format (e.g. as 100 most played songs)
//! to the [`std::io::stdout`]

use std::cmp::Reverse;
use std::collections::HashMap;
use std::fmt::Display;
use std::str::FromStr;

use endsong::prelude::*;
use itertools::Itertools;
use thiserror::Error;

use crate::spaces;

/// Length of indent before a sub-aspect
const INDENT_LENGTH: usize = 4;

/// An enum that is among other things used by functions such as
/// [`top()`] and its derivatives to know whether
/// to print top songs ([`Aspect::Songs`]), albums ([`Aspect::Albums`])
/// or artists ([`Aspect::Artists`])
#[derive(Copy, Clone, Debug)]
pub enum Aspect {
    /// to print top artists
    Artists,
    /// to print top albums
    Albums,
    /// to print top songs, the [`bool`] inside is used
    /// to decide whether or not to
    /// ignore the album (and use only track/artist name) when gathering plays
    /// (false by default when using `.parse()`)
    Songs(bool),
}
impl Display for Aspect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Aspect::Artists => write!(f, "artists"),
            Aspect::Albums => write!(f, "albums"),
            Aspect::Songs(_) => write!(f, "songs"),
        }
    }
}
impl FromStr for Aspect {
    type Err = AspectParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "artist" | "artists" => Ok(Aspect::Artists),
            "album" | "albums" => Ok(Aspect::Albums),
            "song" | "songs" => Ok(Aspect::Songs(false)),
            _ => Err(AspectParseError),
        }
    }
}

/// Error for when the [`FromStr`] impl of [`Aspect`] fails
#[derive(Debug, Error)]
#[error(
    "only \"artist\", \"artists\", \"album\", \"albums\", \"song\" and \"songs\" are valid aspects"
)]
pub struct AspectParseError;

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

/// Trait for handling printing an aspect and its subaspects with playcount etc
pub trait Print: Music {
    /// Prints a specific aspect
    ///
    /// I.e. a song, an album with its songs or an artist with its albums
    fn print(&self, entries: &[SongEntry]);

    /// Prints a specfic aspect in a date range
    ///
    /// I.e. a song, an album with its songs or an artist with its albums
    ///
    /// Basically [`Print::print()`] but with date limitations
    ///
    /// # Panics
    ///
    /// Panics if `start` is after or equal to `end`
    fn print_date(&self, entries: &SongEntries, start: &DateTime<Local>, end: &DateTime<Local>);
}
impl Print for Artist {
    fn print(&self, entries: &[SongEntry]) {
        println!("{} | {} plays", self, gather::plays(entries, self));
        artist(
            entries,
            &gather::albums_from_artist(entries, self),
            INDENT_LENGTH,
        );
    }
    fn print_date(&self, entries: &SongEntries, start: &DateTime<Local>, end: &DateTime<Local>) {
        assert!(start <= end, "Start date is after end date!");
        let entries_within_dates = entries.between(start, end);

        let (start, end) = normalize_dates(entries_within_dates, start, end);

        println!(
            "{} | between {} and {} | {} plays",
            self,
            start.date_naive(),
            end.date_naive(),
            gather::plays(entries_within_dates, self)
        );
        artist(
            entries_within_dates,
            &gather::albums_from_artist(entries_within_dates, self),
            INDENT_LENGTH,
        );
    }
}
impl Print for Album {
    fn print(&self, entries: &[SongEntry]) {
        println!("{} | {} plays", self, gather::plays(entries, self));
        album(&gather::songs_from(entries, self), INDENT_LENGTH);
    }
    fn print_date(&self, entries: &SongEntries, start: &DateTime<Local>, end: &DateTime<Local>) {
        assert!(start <= end, "Start date is after end date!");
        let entries_within_dates = entries.between(start, end);

        let (start, end) = normalize_dates(entries_within_dates, start, end);

        println!(
            "{} | between {} and {} | {} plays",
            self,
            start.date_naive(),
            end.date_naive(),
            gather::plays(entries_within_dates, self)
        );
        album(
            &gather::songs_from(entries_within_dates, self),
            INDENT_LENGTH,
        );
    }
}
impl Print for Song {
    fn print(&self, entries: &[SongEntry]) {
        println!("{} | {} plays", self, gather::plays(entries, self));
    }
    fn print_date(&self, entries: &SongEntries, start: &DateTime<Local>, end: &DateTime<Local>) {
        assert!(start <= end, "Start date is after end date!");
        let entries_within_dates = entries.between(start, end);

        let (start, end) = normalize_dates(entries_within_dates, start, end);

        println!(
            "{} | between {} and {} | {} plays",
            self,
            start.date_naive(),
            end.date_naive(),
            gather::plays(entries_within_dates, self)
        );
    }
}

/// Trait for better display of [durations][TimeDelta]
pub trait DurationUtils {
    /// Returns a string with the duration in the format `HH:MM:SS`
    /// or `MM:SS` (if the duration is less than an hour)
    fn display(&self) -> String;
}
impl DurationUtils for TimeDelta {
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
///    for top albums and [`Aspect::Artists`] for top artists
/// * `num` - number of displayed top aspects.
///   Will automatically change to total number of that aspect if `num` is higher than that
pub fn top(entries: &[SongEntry], asp: Aspect, num: usize) {
    match asp {
        Aspect::Songs(true) => {
            println!("=== TOP {num} SONGS SUMMED ACROSS ALBUMS ===");
            // The album displayed in the parantheses will be the
            // one it has the highest amount of listens from.
            top_helper(gather::songs_summed_across_albums(entries), num);
        }
        Aspect::Songs(false) => {
            println!("=== TOP {num} SONGS ===");
            top_helper(gather::songs(entries), num);
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
///   Will automatically change to total number of that aspect if `num` is higher than that
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
///   Will automatically change to total number of songs from that album
///   if `num` is higher than that
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
        // secondary sorting: by name ascending
        // the key is the tuple (reverse(plays), aspect_name)
        // so it's first compared by the plays in ascending order,
        // and if the plays are equal, it's compared by the name (alphabetical)
        .sorted_unstable_by_key(|(asp, plays)| (Reverse(*plays), asp.clone()))
        // cheap cloning bc Rc::clone() internally
        .collect_vec();
    let length = music_vec.len();

    // if the number of unique aspects is lower than the parsed num
    let max_num: usize = if length < num { length } else { num };

    for (i, (asp, plays)) in music_vec.iter().enumerate().take(max_num) {
        let position = i + 1;
        let indent = spaces((max_num.ilog10() - position.ilog10()) as usize);
        println!("{indent}#{position}: {asp} | {plays} plays");
    }
}

/// Prints a specfic aspect
///
/// Or just import [`Print`] and use `.print()` on the aspect
pub fn aspect<Asp: Print>(entries: &[SongEntry], asp: &Asp) {
    asp.print(entries);
}

/// Prints each [`Album`] of `albums` with the playcount
///
/// Preferably `albums` contains only albums from one artist
fn artist(entries: &[SongEntry], albums: &HashMap<Album, usize>, indent_length: usize) {
    let indent = spaces(indent_length);
    // albums sorted by their playcount descending (primary)
    // and name ascending (secondary) if plays are equal
    let albums_vec: Vec<(&Album, &usize)> = albums
        .iter()
        .sorted_unstable_by_key(|t| (Reverse(t.1), t.0))
        .collect_vec();

    for (alb, plays) in albums_vec {
        println!("{indent}{} | {plays} plays", alb.name);
        album(&gather::songs_from(entries, alb), 2 * indent_length);
    }
}

/// Prints each [`Song`] of `songs` with the playcount
///
/// Preferably `songs` contains only songs from one album
fn album(songs: &HashMap<Song, usize>, indent_length: usize) {
    let indent = spaces(indent_length);
    // songs sorted by their playcount descending (primary)
    // and name ascending (secondary) if plays are equal
    let songs_vec: Vec<(&Song, &usize)> = songs
        .iter()
        .sorted_unstable_by_key(|t| (Reverse(t.1), t.0))
        .collect_vec();

    for (song, plays) in songs_vec {
        println!("{indent}{} | {plays} plays", song.name);
    }
}

/// Prints a specfic aspect in a date range
///
/// Basically [`aspect()`] but with date limitations
///
/// Or just import [`Print`] and use `.print_date()` on the aspect
///
/// # Panics
///
/// Panics if `start` is after or equal to `end`
pub fn aspect_date<Asp: Print>(
    entries: &SongEntries,
    asp: &Asp,
    start: &DateTime<Local>,
    end: &DateTime<Local>,
) {
    asp.print_date(entries, start, end);
}

/// Prints the total time played
#[expect(clippy::missing_panics_doc, reason = "unwrap fine")]
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
#[expect(clippy::cast_precision_loss, reason = "casting i64 to f64")]
#[expect(clippy::cast_possible_wrap, reason = "casting usize to i64")]
pub fn time_played_date(entries: &SongEntries, start: &DateTime<Local>, end: &DateTime<Local>) {
    assert!(start <= end, "Start date is after end date!");
    let duration = gather::total_listening_time(entries.between(start, end));
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
///
/// # Panics
///
/// Will panic if `entries` is empty
fn normalize_dates<'a>(
    entries: &'a [SongEntry],
    start: &'a DateTime<Local>,
    end: &'a DateTime<Local>,
) -> (&'a DateTime<Local>, &'a DateTime<Local>) {
    assert!(
        !entries.is_empty(),
        "there should at least be one SongEntry!"
    );

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

/// Prints each [`SongEntry`] on a specific day
pub fn day(entries: &[SongEntry], date: DateTime<Local>) {
    let day = date.date_naive();
    let day_entries = entries.iter().filter(|e| e.timestamp.date_naive() == day);

    let length = day_entries.clone().count();
    if length == 0 {
        println!("You haven't listened to any songs on {day}!");
        return;
    }
    let time_played = pretty_duration(day_entries.clone().map(|e| e.time_played).sum());
    let sng = if length == 1 { "song" } else { "songs" };
    println!("You've listened to {length} {sng} on {day} for {time_played}!");

    for entry in day_entries {
        println!(
            "{}: {} - {} ({}) for {}s",
            entry.timestamp.time(),
            entry.artist,
            entry.track,
            entry.album,
            entry.time_played.num_seconds()
        );
    }
}

/// Formats [`TimeDelta`] in a convenient way
///
/// I.e. not include hours when only minutes necessary etc.
fn pretty_duration(duration: TimeDelta) -> String {
    let hours = duration.num_hours();
    let minutes = duration.num_minutes();

    if minutes == 0 || minutes == 1 {
        return format!("{} seconds", duration.num_seconds());
    }

    if hours > 0 {
        let hrs = if hours == 1 { "hour" } else { "hours" };
        let remaining_minutes = minutes % (60 * hours);
        // yes, I know ignoring 1min :)
        if remaining_minutes == 0 || remaining_minutes == 1 {
            return format!("{hours} {hrs}");
        }
        return format!("{hours} {hrs} and {remaining_minutes} minutes");
    }

    format!("{minutes} minutes")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests [`pretty_duration`]
    #[test]
    fn pretty_dur() {
        let thirty_secs = TimeDelta::seconds(30);
        assert_eq!(pretty_duration(thirty_secs), "30 seconds");

        let one_half_min = TimeDelta::seconds(90);
        assert_eq!(pretty_duration(one_half_min), "90 seconds");

        let two_mins = TimeDelta::minutes(2);
        assert_eq!(pretty_duration(two_mins), "2 minutes");

        let fifty_nine_mins = TimeDelta::minutes(59);
        assert_eq!(pretty_duration(fifty_nine_mins), "59 minutes");

        let one_hour = TimeDelta::hours(1);
        assert_eq!(pretty_duration(one_hour), "1 hour");

        let one_half_hours = TimeDelta::minutes(90);
        assert_eq!(pretty_duration(one_half_hours), "1 hour and 30 minutes");

        let three_half_hours = TimeDelta::minutes(3 * 60 + 30);
        assert_eq!(pretty_duration(three_half_hours), "3 hours and 30 minutes");

        let five_days = TimeDelta::days(5);
        assert_eq!(pretty_duration(five_days), "120 hours");
    }
}
