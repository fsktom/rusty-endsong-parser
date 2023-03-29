use super::{create_plot, find_dates};
use crate::display::date;
use crate::types::{Artist, SongEntries};

/// Creates a plot of the absolute amount of plays of an [`Artist`]
///
/// Opens the plot in the browser
pub fn artist(entries: &SongEntries, art: &Artist) {
    let mut times = Vec::<i64>::new();
    let mut plays = Vec::<usize>::new();

    let dates = find_dates(entries, art, true);

    let start = dates.first().unwrap();

    for date in &dates {
        times.push(date.timestamp());
        plays.push(date::gather_artist(entries, art, start, date));
    }

    create_plot(times, plays, art.name.as_str());
}
