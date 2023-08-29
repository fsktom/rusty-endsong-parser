//! Module for creating traces used in [`plot`][crate::plot]

use endsong::prelude::*;
use plotly::{Scatter, Trace};

/// Formats date for x-axis to `%Y-%m-%d %H:%M`
///
/// I.e. "2016-09-01 15:06"
fn format_date(date: &DateTime<Local>) -> String {
    date.format("%Y-%m-%d %H:%M").to_string()
}

/// Creates a trace of the absolute amount of plays
///
/// Creates an empty trace if `aspect` is not in `entries`
#[must_use]
pub fn absolute<Asp: Music>(entries: &SongEntries, aspect: &Asp) -> (Box<dyn Trace>, String) {
    let mut times = Vec::<String>::with_capacity(entries.len());
    let mut plays = Vec::<usize>::with_capacity(entries.len());

    // since each date represents a single listen, we can just count up
    let mut aspect_plays = 0;

    for entry in entries.iter().filter(|entry| aspect.is_entry(entry)) {
        aspect_plays += 1;
        times.push(format_date(&entry.timestamp));
        plays.push(aspect_plays);
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

    use super::format_date;

    /// Creates a trace of the amount of plays of an [`Music`] relative to all plays
    ///
    /// Creates an empty trace if `aspect` is not in `entries`
    #[must_use]
    pub fn to_all<Asp: Music>(entries: &SongEntries, aspect: &Asp) -> (Box<dyn Trace>, String) {
        let mut times = Vec::<String>::with_capacity(entries.len());
        // percentages relative to the sum of all plays
        let mut plays = Vec::<f64>::with_capacity(entries.len());

        let mut aspect_plays = 0.0;
        let mut all_plays = 0.0;

        // the plot should start at the first time the aspect is played
        let mut aspect_found = false;

        for entry in entries.iter() {
            all_plays += 1.0;

            if aspect.is_entry(entry) {
                aspect_found = true;
                aspect_plays += 1.0;
            }
            if aspect_found {
                times.push(format_date(&entry.timestamp));
                // *100 so that the percentage is easier to read...
                plays.push(100.0 * (aspect_plays / all_plays));
            }
        }

        let title = format!("{aspect} | relative to all plays");
        let trace = Scatter::new(times, plays).name(&title);
        (trace, title)
    }

    /// Creates a plot of the amount of plays of an [`Album`] or [`Song`]
    /// relative to total plays of the corresponding [`Artist`]
    ///
    /// Creates an empty trace if `aspect` is not in `entries`
    #[must_use]
    pub fn to_artist<Asp: AsRef<Album> + Music>(
        entries: &SongEntries,
        aspect: &Asp,
    ) -> (Box<dyn Trace>, String) {
        let artist = &aspect.as_ref().artist;

        let mut times = Vec::<String>::new();
        // percentages relative to the sum of respective artist plays
        let mut plays = Vec::<f64>::new();

        let mut aspect_plays = 0.0;
        let mut artist_plays = 0.0;

        // the plot should start at the first time the aspect is played
        let mut aspect_found = false;

        for entry in entries.iter().filter(|entry| artist.is_entry(entry)) {
            artist_plays += 1.0;

            if aspect.is_entry(entry) {
                aspect_found = true;
                aspect_plays += 1.0;
            }

            if aspect_found {
                times.push(format_date(&entry.timestamp));
                // *100 so that the percentage is easier to read...
                plays.push(100.0 * (aspect_plays / artist_plays));
            }
        }

        let title = format!("{aspect} | relative to the artist");
        let trace = Scatter::new(times, plays).name(&title);
        (trace, title)
    }

    /// Creates a plot of the amount of plays of a [`Song`]
    /// relative to total plays of the corresponding [`Album`]
    ///
    /// Creates an empty trace if `song` is not in `entries`
    #[must_use]
    pub fn to_album(entries: &SongEntries, song: &Song) -> (Box<dyn Trace>, String) {
        let album = &song.album;

        let mut times = Vec::<String>::new();
        // percentages relative to the sum of respective album plays
        let mut plays = Vec::<f64>::new();

        let mut song_plays = 0.0;
        let mut album_plays = 0.0;

        // the plot should start at the first time the aspect is played
        let mut song_found = false;

        for entry in entries.iter().filter(|entry| album.is_entry(entry)) {
            album_plays += 1.0;

            if song.is_entry(entry) {
                song_found = true;
                song_plays += 1.0;
            }

            if song_found {
                times.push(format_date(&entry.timestamp));
                // *100 so that the percentage is easier to read...
                plays.push(100.0 * (song_plays / album_plays));
            }
        }

        let title = format!("{song} | relative to the album");
        let trace = Scatter::new(times, plays).name(&title);
        (trace, title)
    }
}
