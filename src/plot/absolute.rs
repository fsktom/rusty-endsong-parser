use plotly::{Scatter, Trace};

use super::find_dates;
use crate::display::date;
use crate::types::{Music, SongEntries};

/// Creates a trace of the absolute amount of plays
pub fn aspect<Asp: Music>(entries: &SongEntries, aspect: &Asp) -> (Box<dyn Trace>, String) {
    let mut times = Vec::<i64>::new();
    let mut plays = Vec::<usize>::new();

    let dates = find_dates(entries, aspect, false);

    let start = dates.first().unwrap();

    for date in &dates {
        times.push(date.timestamp());
        plays.push(date::gather_plays(entries, aspect, start, date));
    }

    let title = format!("{aspect}");
    let trace = Scatter::new(times, plays).name(&title);
    (trace, title)
}
