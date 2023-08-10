//! Module for creating traces used in [`plot`][crate::plot]

use endsong::prelude::*;
use plotly::{Scatter, Trace};

/// Formats date for x-axis`%Y-%m-%d %H:%M`
///
/// To something like "2016-09-01 15:06"
fn format_date(date: &DateTime<Local>) -> String {
    date.format("%Y-%m-%d %H:%M").to_string()
}

/// Creates a trace of the absolute amount of plays
pub fn absolute<Asp: Music>(entries: &SongEntries, aspect: &Asp) -> (Box<dyn Trace>, String) {
    let mut times = Vec::<String>::with_capacity(entries.len());
    let mut plays = Vec::<usize>::with_capacity(entries.len());

    // since each date represents a single listen, we can just count up
    let mut aspect_plays = 1;

    for entry in entries.iter().filter(|entry| aspect.is_entry(entry)) {
        times.push(format_date(&entry.timestamp));
        plays.push(aspect_plays);
        aspect_plays += 1;
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
    use itertools::Itertools;
    use plotly::{Scatter, Trace};

    use super::format_date;

    /// Creates a trace of the amount of plays of an [`Music`] relative to all plays
    pub fn to_all<Asp: Music>(entries: &SongEntries, aspect: &Asp) -> (Box<dyn Trace>, String) {
        let mut times = Vec::<String>::with_capacity(entries.len());
        // percentages relative to the sum of all plays
        let mut plays = Vec::<f64>::with_capacity(entries.len());

        let first_aspect = entries.iter().find(|entry| aspect.is_entry(entry)).unwrap();
        let first_aspect_occ = entries
            .binary_search_by_key(&first_aspect.timestamp, |entry| entry.timestamp)
            .unwrap();

        // since each date represents a single listen, we can just count up
        let mut aspect_plays = 1.0;
        #[allow(clippy::cast_precision_loss)]
        let mut all_plays = entries[..=first_aspect_occ].len() as f64;

        for entry in &entries[first_aspect_occ + 1..] {
            times.push(format_date(&entry.timestamp));
            // *100 so that the percentage is easier to read...
            plays.push(100.0 * (aspect_plays / all_plays));
            all_plays += 1.0;
            if aspect.is_entry(entry) {
                aspect_plays += 1.0;
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
        // since it's relative to the artist, going through artist entries is enough
        let artist_entries = entries
            .iter()
            .filter(|entry| aspect.as_ref().is_entry(entry))
            .collect_vec();

        let mut times = Vec::<String>::with_capacity(artist_entries.len());
        // percentages relative to the sum of respective artist plays
        let mut plays = Vec::<f64>::with_capacity(artist_entries.len());

        let first_aspect = artist_entries
            .iter()
            .find(|entry| aspect.is_entry(entry))
            .unwrap();
        let first_aspect_occ = artist_entries
            .binary_search_by_key(&first_aspect.timestamp, |entry| entry.timestamp)
            .unwrap();

        let mut aspect_plays = 1.0;
        #[allow(clippy::cast_precision_loss)]
        let mut artist_plays = artist_entries[..=first_aspect_occ].len() as f64;

        for entry in &artist_entries[first_aspect_occ + 1..] {
            times.push(format_date(&entry.timestamp));

            // *100 so that the percentage is easier to read...
            plays.push(100.0 * (aspect_plays / artist_plays));

            artist_plays += 1.0;
            if aspect.is_entry(entry) {
                aspect_plays += 1.0;
            }
        }

        let title = format!("{aspect} | relative to the artist");
        let trace = Scatter::new(times, plays).name(&title);
        (trace, title)
    }

    /// Creates a plot of the amount of plays of a [`Song`]
    /// relative to total plays of the corresponding [`Album`]
    pub fn to_album(entries: &SongEntries, song: &Song) -> (Box<dyn Trace>, String) {
        // since it's relative to the album, going through album entries is enough
        let album_entries = entries
            .iter()
            .filter(|entry| song.album.is_entry(entry))
            .collect_vec();

        let mut times = Vec::<String>::with_capacity(album_entries.len());
        // percentages relative to the sum of respective album plays
        let mut plays = Vec::<f64>::with_capacity(album_entries.len());

        let first_song = album_entries
            .iter()
            .find(|entry| song.is_entry(entry))
            .unwrap();
        let first_song_occ = album_entries
            .binary_search_by_key(&first_song.timestamp, |entry| entry.timestamp)
            .unwrap();

        // since each date represents a single listen, we can just count up
        let mut song_plays = 1.0;
        #[allow(clippy::cast_precision_loss)]
        let mut album_plays = album_entries[..=first_song_occ].len() as f64;

        for entry in &album_entries[first_song_occ + 1..] {
            times.push(format_date(&entry.timestamp));

            // *100 so that the percentage is easier to read...
            plays.push(100.0 * (song_plays / album_plays));

            album_plays += 1.0;
            if song.is_entry(entry) {
                song_plays += 1.0;
            }
        }

        let title = format!("{song} | relative to the album");
        let trace = Scatter::new(times, plays).name(&title);
        (trace, title)
    }
}
