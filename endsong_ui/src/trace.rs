//! Module for creating traces used in [`plot`][crate::plot]

use endsong::prelude::*;
use plotly::{Scatter, Trace};

/// Returns the dates of all occurrences of the `aspect`
///
/// * `add_now` - with this set to true, it will put the current time as the last date,
/// otherwise it will be the last occurrence of `aspect`
fn find_dates<Asp: Music>(entries: &[SongEntry], aspect: &Asp, add_now: bool) -> Vec<DateTime<Tz>> {
    let mut dates = Vec::<DateTime<Tz>>::new();

    for entry in entries {
        if aspect.is_entry(entry) {
            dates.push(entry.timestamp);
        }
    }

    if add_now {
        dates.push(
            crate::LOCATION_TZ
                .timestamp_millis_opt(chrono::offset::Local::now().timestamp_millis())
                .unwrap(),
        );
    }

    // should be sorted because &[SongEntries] should have been sorted at the beginning
    dates
}

/// Creates a trace of the absolute amount of plays
pub fn absolute<Asp: Music>(entries: &SongEntries, aspect: &Asp) -> (Box<dyn Trace>, String) {
    let mut times = Vec::<i64>::new();
    let mut plays = Vec::<usize>::new();

    let dates = find_dates(entries, aspect, false);

    // since each date represents a single listen, we can just count up
    let mut amount_of_plays = 1;

    for date in &dates {
        times.push(date.timestamp());
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
    use plotly::{Scatter, Trace};

    use super::find_dates;

    /// Creates a trace of the amount of plays of an [`Music`] relative to all plays
    pub fn to_all<Asp: Music>(entries: &SongEntries, aspect: &Asp) -> (Box<dyn Trace>, String) {
        let mut times = Vec::<i64>::new();
        // percentages relative to the sum of all plays
        let mut plays = Vec::<f64>::new();

        // TODO!
        // each data point lies at the occurrence -> looks weird when you haven't listened in a long time
        // maybe make it so there's at least a data point once a week?
        let dates = find_dates(entries, aspect, false);

        let sum_start = &entries.first_date();

        // since each date represents a single listen, we can just count up
        let mut amount_of_plays = 1.0;

        #[allow(clippy::cast_precision_loss)]
        for date in &dates {
            times.push(date.timestamp());
            let sum_of_all_plays = gather::all_plays(entries.between(sum_start, date)) as f64;
            // *100 so that the percentage is easier to read...
            plays.push(100.0 * (amount_of_plays / sum_of_all_plays));
            amount_of_plays += 1.0;
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
        let mut times = Vec::<i64>::new();
        // percentages relative to the sum of respective artist plays
        let mut plays = Vec::<f64>::new();

        // TODO!
        // each data point lies at the occurrence -> looks weird when you haven't listened in a long time
        // maybe make it so there's at least a data point once a week?
        let dates = find_dates(entries, aspect, false);
        let artist_dates = find_dates(entries, aspect.as_ref(), false);

        // since each date represents a single listen, we can just count up
        let mut amount_of_plays = 1.0;

        #[allow(clippy::cast_precision_loss)]
        for date in &dates {
            times.push(date.timestamp());

            let end = artist_dates.binary_search(date).unwrap();
            let sum_of_artist_plays = artist_dates[..=end].len() as f64;

            // *100 so that the percentage is easier to read...
            plays.push(100.0 * (amount_of_plays / sum_of_artist_plays));
            amount_of_plays += 1.0;
        }

        let title = format!("{aspect} | relative to the artist");
        let trace = Scatter::new(times, plays).name(&title);
        (trace, title)
    }

    /// Creates a plot of the amount of plays of a [`Song`]
    /// relative to total plays of the corresponding [`Album`]
    pub fn to_album(entries: &SongEntries, aspect: &Song) -> (Box<dyn Trace>, String) {
        let mut times = Vec::<i64>::new();
        // percentages relative to the sum of respective album plays
        let mut plays = Vec::<f64>::new();

        // TODO!
        // each data point lies at the occurrence -> looks weird when you haven't listened in a long time
        // maybe make it so there's at least a data point once a week?
        let dates = find_dates(entries, aspect, false);
        let album_dates = find_dates(entries, &aspect.album, false);

        // since each date represents a single listen, we can just count up
        let mut amount_of_plays = 1.0;

        #[allow(clippy::cast_precision_loss)]
        for date in &dates {
            times.push(date.timestamp());

            let end = album_dates.binary_search(date).unwrap();
            let sum_of_album_plays = album_dates[..=end].len() as f64;

            // *100 so that the percentage is easier to read...
            plays.push(100.0 * (amount_of_plays / sum_of_album_plays));
            amount_of_plays += 1.0;
        }

        let title = format!("{aspect} | relative to the album");
        let trace = Scatter::new(times, plays).name(&title);
        (trace, title)
    }
}
