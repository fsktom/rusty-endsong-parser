//! Module for creating traces used in [`plot`][crate::plot]

use endsong::prelude::*;
use itertools::Itertools;
use plotly::{Scatter, Trace};

/// Returns the dates of all occurrences of the `aspect` in ascending order
fn find_dates<Asp: Music>(entries: &[SongEntry], aspect: &Asp) -> Vec<DateTime<Tz>> {
    entries
        .iter()
        .filter(|entry| aspect.is_entry(entry))
        .map(|entry| entry.timestamp)
        .collect_vec()
}

/// Generates a [`Vec`] of [`DateTime`]s `resolution` apart going from
/// from `first` to the last entry in `entries` in ascending order
///
/// Recommended `resolution` is `Duration::days(1)`
fn generate_dates(
    entries: &[SongEntry],
    first: DateTime<Tz>,
    resolution: Duration,
) -> Vec<DateTime<Tz>> {
    let mut dates = Vec::<DateTime<Tz>>::new();

    let mut head = first + resolution;
    let last = entries.last().unwrap().timestamp;
    while head < last {
        dates.push(head);
        head += resolution;
    }

    dates
}

/// Formats date for x-axis`%Y-%m-%d %H:%M`
///
/// To something like "2016-09-01 15:06"
fn format_date(date: &DateTime<Tz>) -> String {
    date.format("%Y-%m-%d %H:%M").to_string()
}

/// Creates a trace of the absolute amount of plays
pub fn absolute<Asp: Music>(entries: &SongEntries, aspect: &Asp) -> (Box<dyn Trace>, String) {
    let mut times = Vec::<String>::new();
    let mut plays = Vec::<usize>::new();

    let dates = find_dates(entries, aspect);

    // since each date represents a single listen, we can just count up
    let mut amount_of_plays = 1;

    for date in &dates {
        times.push(format_date(date));
        plays.push(amount_of_plays);
        amount_of_plays += 1;
    }

    let title = format!("{aspect}");
    let trace = Scatter::new(times, plays).name(&title);
    (trace, title)
}

/// Module for relative traces
///
/// Either to all plays, the artist or the album
pub mod relative {
    use endsong::prelude::*;
    use itertools::PeekingNext;
    use plotly::{Scatter, Trace};

    use super::find_dates;
    use super::format_date;
    use super::generate_dates;

    /// Creates a trace of the amount of plays of an [`Music`] relative to all plays
    pub fn to_all<Asp: Music>(entries: &SongEntries, aspect: &Asp) -> (Box<dyn Trace>, String) {
        let mut times = Vec::<String>::new();
        // percentages relative to the sum of all plays
        let mut plays = Vec::<f64>::new();

        let dates = find_dates(entries, aspect);

        // for more resolution, we add an entry for each day
        let mut all_dates = generate_dates(entries, *dates.first().unwrap(), Duration::days(1));
        all_dates.extend_from_slice(&dates);
        all_dates.sort_unstable();

        let mut dates_iter = dates.iter();
        dates_iter.next();

        let sum_start = &entries.first_date();

        // since each date represents a single listen, we can just count up
        let mut amount_of_plays = 1.0;

        #[allow(clippy::cast_precision_loss)]
        for date in &all_dates {
            times.push(format_date(date));
            let sum_of_all_plays = gather::all_plays(entries.between(sum_start, date)) as f64;
            // *100 so that the percentage is easier to read...
            plays.push(100.0 * (amount_of_plays / sum_of_all_plays));

            if dates_iter.peeking_next(|d| *d == date).is_some() {
                amount_of_plays += 1.0;
            }
        }

        let title = format!("{aspect} | relative to all plays");
        let trace = Scatter::new(times, plays).name(&title);
        (trace, title)
    }

    /// Creates a plot of the amount of plays of an [`Album`] or [`Song`]
    /// relative to total plays of the corresponding [`Artist`]
    pub fn to_artist<Asp: AsRef<Artist> + Music>(
        entries: &SongEntries,
        aspect: &Asp,
    ) -> (Box<dyn Trace>, String) {
        let mut times = Vec::<String>::new();
        // percentages relative to the sum of respective artist plays
        let mut plays = Vec::<f64>::new();

        let dates = find_dates(entries, aspect);
        let artist_dates = find_dates(entries, aspect.as_ref());

        // for highest resolution, we take each artist occurrence (since it's relative to the artist)
        let first_occ = artist_dates.binary_search(dates.first().unwrap()).unwrap();

        let mut dates_iter = dates.iter();
        dates_iter.next();

        // since each date represents a single listen, we can just count up
        let mut amount_of_plays = 1.0;

        #[allow(clippy::cast_precision_loss)]
        for date in &artist_dates[first_occ..] {
            times.push(format_date(date));

            let end = match artist_dates.binary_search(date) {
                Ok(i) => i,
                Err(i) => i - 1,
            };
            let sum_of_artist_plays = artist_dates[..=end].len() as f64;

            // *100 so that the percentage is easier to read...
            plays.push(100.0 * (amount_of_plays / sum_of_artist_plays));

            if dates_iter.peeking_next(|d| *d == date).is_some() {
                amount_of_plays += 1.0;
            }
        }

        let title = format!("{aspect} | relative to the artist");
        let trace = Scatter::new(times, plays).name(&title);
        (trace, title)
    }

    /// Creates a plot of the amount of plays of a [`Song`]
    /// relative to total plays of the corresponding [`Album`]
    pub fn to_album(entries: &SongEntries, aspect: &Song) -> (Box<dyn Trace>, String) {
        let mut times = Vec::<String>::new();
        // percentages relative to the sum of respective album plays
        let mut plays = Vec::<f64>::new();

        let dates = find_dates(entries, aspect);
        let album_dates = find_dates(entries, &aspect.album);

        // for the highest resolution, we take each album occurrence (since it's relative to the album)
        let first_occ = album_dates.binary_search(dates.first().unwrap()).unwrap();

        let mut dates_iter = dates.iter();
        dates_iter.next();

        // since each date represents a single listen, we can just count up
        let mut amount_of_plays = 1.0;

        #[allow(clippy::cast_precision_loss)]
        for date in &album_dates[first_occ..] {
            times.push(format_date(date));

            let end = album_dates.binary_search(date).unwrap();
            let sum_of_album_plays = album_dates[..=end].len() as f64;

            // *100 so that the percentage is easier to read...
            plays.push(100.0 * (amount_of_plays / sum_of_album_plays));

            if dates_iter.peeking_next(|d| *d == date).is_some() {
                amount_of_plays += 1.0;
            }
        }

        let title = format!("{aspect} | relative to the album");
        let trace = Scatter::new(times, plays).name(&title);
        (trace, title)
    }
}
