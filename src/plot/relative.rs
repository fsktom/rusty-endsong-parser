use super::{create_plot, find_dates};
use crate::display::date;
use crate::types::{Artist, SongEntries};

/// Creates a plot of the amount of plays of an [`Artist`] relative to all plays
///
/// Opens the plot in the browser
pub fn artist(entries: &SongEntries, art: &Artist) {
    let mut times = Vec::<i64>::new();
    // percentages relative to the sum of all plays
    let mut plays = Vec::<f64>::new();

    let dates = find_dates(entries, art, true);

    let start = dates.first().unwrap();
    let sum_start = &entries.first_date();

    #[allow(clippy::cast_precision_loss)]
    for date in &dates {
        times.push(date.timestamp());
        let sum_of_plays = date::gather_artist(entries, art, start, date) as f64;
        let sum_of_all_plays = date::sum_plays(entries, sum_start, date) as f64;
        plays.push(sum_of_plays / sum_of_all_plays);
    }

    create_plot(times, plays, format!("{art} - relative").as_str());
}
