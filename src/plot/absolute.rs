use plotly::{Scatter, Trace};

use super::find_dates;
use crate::types::{Music, SongEntries};

/// Creates a trace of the absolute amount of plays
pub fn aspect<Asp: Music>(entries: &SongEntries, aspect: &Asp) -> (Box<dyn Trace>, String) {
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
