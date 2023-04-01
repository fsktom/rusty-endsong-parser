use plotly::{Scatter, Trace};

use super::find_dates;
use crate::display::date;
use crate::types::{HasArtist, Music, Song, SongEntries};

/// Creates a trace of the amount of plays of an [`Music`] relative to all plays
pub fn to_all<Asp: Music>(entries: &SongEntries, aspect: &Asp) -> (Box<dyn Trace>, String) {
    let mut times = Vec::<i64>::new();
    // percentages relative to the sum of all plays
    let mut plays = Vec::<f64>::new();

    // TODO!
    // each data point lies at the occurrence -> looks weird when you haven't listened in a long time
    // maybe make it so there's at least a data point once a week?
    let dates = find_dates(entries, aspect, false);

    let start = dates.first().unwrap();
    let sum_start = &entries.first_date();

    #[allow(clippy::cast_precision_loss)]
    for date in &dates {
        times.push(date.timestamp());
        let sum_of_plays = date::gather_plays(entries, aspect, start, date) as f64;
        let sum_of_all_plays = date::sum_plays(entries, sum_start, date) as f64;
        // *100 so that the percentage is easier to read...
        plays.push(100.0 * (sum_of_plays / sum_of_all_plays));
    }

    let title = format!("{aspect} | relative to all plays");
    let trace = Scatter::new(times, plays).name(&title);
    (trace, title)
}

/// Creates a plot of the amount of plays of an [`Album`][crate::types::Album] or [`Song`]
/// relative to total plays of the affiated [`Artist`][crate::types::Artist]
pub fn to_artist<Asp: HasArtist>(entries: &SongEntries, aspect: &Asp) -> (Box<dyn Trace>, String) {
    let mut times = Vec::<i64>::new();
    // percentages relative to the sum of all plays
    let mut plays = Vec::<f64>::new();

    // TODO!
    // each data point lies at the occurrence -> looks weird when you haven't listened in a long time
    // maybe make it so there's at least a data point once a week?
    let dates = find_dates(entries, aspect, false);

    let start = dates.first().unwrap();
    let sum_start = &entries.first_date();

    #[allow(clippy::cast_precision_loss)]
    for date in &dates {
        times.push(date.timestamp());
        let sum_of_plays = date::gather_plays(entries, aspect, start, date) as f64;
        let sum_of_artist_plays =
            date::gather_plays(entries, aspect.artist(), sum_start, date) as f64;
        // *100 so that the percentage is easier to read...
        plays.push(100.0 * (sum_of_plays / sum_of_artist_plays));
    }

    let title = format!("{aspect} | relative to the artist");
    let trace = Scatter::new(times, plays).name(&title);
    (trace, title)
}

/// Creates a plot of the amount of plays of a [`Song`]
/// relative to total plays of the affiated [`Album`][crate::types::Album]
pub fn to_album(entries: &SongEntries, aspect: &Song) -> (Box<dyn Trace>, String) {
    let mut times = Vec::<i64>::new();
    // percentages relative to the sum of all plays
    let mut plays = Vec::<f64>::new();

    // TODO!
    // each data point lies at the occurrence -> looks weird when you haven't listened in a long time
    // maybe make it so there's at least a data point once a week?
    let dates = find_dates(entries, aspect, false);

    let start = dates.first().unwrap();
    let sum_start = &entries.first_date();

    #[allow(clippy::cast_precision_loss)]
    for date in &dates {
        times.push(date.timestamp());
        let sum_of_plays = date::gather_plays(entries, aspect, start, date) as f64;
        let sum_of_album_plays = date::gather_plays(entries, &aspect.album, sum_start, date) as f64;
        // *100 so that the percentage is easier to read...
        plays.push(100.0 * (sum_of_plays / sum_of_album_plays));
    }

    let title = format!("{aspect} | relative to the album");
    let trace = Scatter::new(times, plays).name(&title);
    (trace, title)
}
