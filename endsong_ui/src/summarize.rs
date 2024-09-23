//! Generates HTML page summaries of aspects

use std::collections::HashMap;
use std::rc::Rc;

use endsong::prelude::*;
use itertools::Itertools;
use rinja::Template;

/// Used for generating an HTML summary page of an [`Artist`]
#[derive(Template)]
#[template(path = "artist.html", print = "none")]
struct ArtistSummary {
    /// artist name
    name: Rc<str>,
    /// number of top songs/albums to be displayed
    top: usize,
    /// array of top song names with their playcount
    songs: Vec<(Rc<str>, usize)>,
    /// array of top album names with their playcount
    albums: Vec<(Rc<str>, usize)>,
    /// number of this artist's plays
    plays: usize,
    /// % of total plays
    percentage_of_plays: String,
    /// Date of first listen
    first_listen: DateTime<Local>,
    /// Date of last listen
    last_listen: DateTime<Local>,
    /// Current time
    now: DateTime<Local>,
    /// Names of the files used for the [`SongEntries`]
    filenames: Vec<String>,
}

/// Generates an HTML summary page of an [`Artist`]
#[expect(clippy::missing_panics_doc, reason = "placeholder")]
#[expect(clippy::cast_precision_loss, reason = "necessary for %")]
pub fn artist(entries: &SongEntries, artist: &Artist) {
    let top = 10;

    let song_map = gather::songs_from_artist_summed_across_albums(entries, artist);
    let songs = get_sorted_playcount_list(song_map, top);

    let album_map = gather::albums_from_artist(entries, artist);
    let albums = get_sorted_playcount_list(album_map, top);

    let plays = gather::plays(entries, artist);
    let percentage_of_plays = format!("{:.2}", (plays as f64 / entries.len() as f64) * 100.0);

    let first_listen = entries
        .iter()
        .find(|entry| artist.is_entry(entry))
        .unwrap()
        .timestamp;
    let last_listen = entries
        .iter()
        .rev()
        .find(|entry| artist.is_entry(entry))
        .unwrap()
        .timestamp;

    let now = Local::now();

    let filenames = entries
        .files_used
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();

    let page = ArtistSummary {
        name: std::rc::Rc::clone(&artist.name),
        top,
        songs,
        albums,
        plays,
        percentage_of_plays,
        first_listen,
        last_listen,
        now,
        filenames,
    };
    std::fs::create_dir_all("summaries").unwrap();
    let path = format!("summaries/{} summary.html", artist.name);
    std::fs::write(&path, page.render().unwrap()).unwrap();
    // std::process::Command::new("open")
    //     .arg(&path)
    //     .output()
    //     .unwrap();
}

/// Makes a list of aspects with their total playcount sorted by their
/// playcount descending and then alphabetically
///
/// Use with maps gotten through [`gather`] functions
fn get_sorted_playcount_list<Asp: Music>(
    map: HashMap<Asp, usize>,
    top: usize,
) -> Vec<(Rc<str>, usize)> {
    map.into_iter()
        .sorted_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)))
        .take(top)
        .map(|(asp, plays)| (asp.name(), plays))
        .collect()
}
