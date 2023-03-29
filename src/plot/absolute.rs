use super::create_plot;
use crate::display::date;
use crate::types::{Artist, Music, SongEntries, SongEntry};
use crate::ui::user_input_date_parser;

use chrono::DateTime;
use chrono_tz::Tz;

/// Creates a plot of the absolute amount of plays of an [`Artist`]
///
/// Opens the plot in the browser
pub fn artist(entries: &SongEntries, art: &Artist) {
    let mut times = Vec::<i64>::new();
    let mut plays = Vec::<usize>::new();

    let mut dates = find_artist_dates(entries, art);
    dates.sort();

    let start = dates.first().unwrap();

    for date in &dates {
        times.push(date.timestamp());
        plays.push(date::gather_artist(entries, art, start, date));
    }

    create_plot(times, plays, art.name.as_str());
}

/// Used by [`artist()`] to get the dates of all of its occurrences
fn find_artist_dates(entries: &Vec<SongEntry>, art: &Artist) -> Vec<DateTime<Tz>> {
    let mut dates = Vec::<DateTime<Tz>>::new();

    for entry in entries {
        if art.is_entry(entry) {
            dates.push(entry.timestamp);
        }
    }

    dates.push(user_input_date_parser("now").unwrap());

    dates
}
