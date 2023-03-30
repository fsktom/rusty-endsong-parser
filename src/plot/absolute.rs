use super::{create_plot, find_dates};
use crate::display::date;
use crate::types::{Music, SongEntries};

/// Creates a plot of the absolute amount of plays
///
/// Opens the plot in the browser
pub fn aspect<Asp: Music>(entries: &SongEntries, aspect: &Asp) {
    let mut times = Vec::<i64>::new();
    let mut plays = Vec::<usize>::new();

    let dates = find_dates(entries, aspect, true);

    let start = dates.first().unwrap();

    for date in &dates {
        times.push(date.timestamp());
        plays.push(date::gather_plays(entries, aspect, start, date));
    }

    let title = format!("{aspect}");
    create_plot(times, plays, &title);
}
