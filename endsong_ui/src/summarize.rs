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
    /// Artist name
    name: Rc<str>,
    /// Number of top songs/albums to be displayed
    top: usize,
    /// Array of top song names with their playcount
    songs: Vec<(Rc<str>, usize)>,
    /// Array of top album names with their playcount
    albums: Vec<(Rc<str>, usize)>,
    /// Count of this artist's plays
    plays: usize,
    /// Total time listend to this artist
    time_played: TimeDelta,
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
    /// First `top` listens of that artist
    first_listens: Vec<SongEntry>,
    /// Last `top` listens of that artist
    last_listens: Vec<SongEntry>,
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

    let time_played = entries
        .iter()
        .filter(|e| artist.is_entry(e))
        .map(|e| e.time_played)
        .sum();

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

    let first_listens = entries
        .iter()
        .filter(|e| artist.is_entry(e))
        .take(top)
        .cloned()
        .collect();
    let last_listens = entries
        .iter()
        .rev()
        .filter(|e| artist.is_entry(e))
        .take(top)
        .cloned()
        .collect();

    let page = ArtistSummary {
        name: std::rc::Rc::clone(&artist.name),
        top,
        songs,
        albums,
        plays,
        percentage_of_plays,
        time_played,
        first_listen,
        last_listen,
        now,
        filenames,
        first_listens,
        last_listens,
    };
    write_and_open_summary(page, &artist.name);
}

/// Creates the summary .html in the plots/ folder and opens it in the browser
///
/// Compare with [`crate::plot::write_and_open_plot`]
fn write_and_open_summary<S: Template>(summary: S, name: &str) {
    std::fs::create_dir_all("summaries").unwrap();

    let title = crate::normalize_path(&format!("{name}_summary"));

    match std::env::consts::OS {
        // see https://github.com/plotly/plotly.rs/issues/132#issuecomment-1488920563
        "windows" => {
            let path = format!(
                "{}\\summaries\\{}.html",
                std::env::current_dir().unwrap().display(),
                title
            );
            std::fs::write(&path, summary.render().unwrap()).unwrap();
            std::process::Command::new("explorer")
                .arg(&path)
                .output()
                .unwrap();
        }
        "macos" => {
            let path = format!(
                "{}/summaries/{}.html",
                std::env::current_dir().unwrap().display(),
                title
            );
            std::fs::write(&path, summary.render().unwrap()).unwrap();
            std::process::Command::new("open")
                .arg(&path)
                .output()
                .unwrap();
        }
        _ => {
            let path = format!(
                "{}/summaries/{}.html",
                std::env::current_dir().unwrap().display(),
                title
            );
            std::fs::write(&path, summary.render().unwrap()).unwrap();

            match std::env::var("BROWSER") {
                Ok(browser) => {
                    std::process::Command::new(browser)
                        .arg(&path)
                        .output()
                        .unwrap();
                }
                Err(_) => {
                    eprintln!("Your BROWSER environmental variable is not set!");
                }
            }
        }
    };
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

/// Custom [`rinja`] filters used by templates here
///
/// See <https://rinja.readthedocs.io/en/stable/filters.html#custom-filters>
mod filters {
    use endsong::prelude::*;

    /// Pretty formats a [`DateTime`]
    #[expect(clippy::unnecessary_wraps, reason = "rinja required output type")]
    pub fn pretty_date(date: &DateTime<Local>) -> rinja::Result<String> {
        Ok(format!(
            "{:04}-{:02}-{:02} {:02}:{:02}",
            date.year(),
            date.month(),
            date.day(),
            date.hour(),
            date.minute(),
        ))
    }

    /// Pretty formats a [`TimeDelta`] in a reasonable way
    ///
    /// ```"_d _h _m"```
    #[expect(clippy::unnecessary_wraps, reason = "rinja required output type")]
    pub fn pretty_duration(duration: &TimeDelta) -> rinja::Result<String> {
        let days = duration.num_days();
        let hours = duration.num_hours();
        let minutes = duration.num_minutes();

        if minutes == 0 || minutes == 1 {
            return Ok(format!("{}s", duration.num_seconds()));
        }

        if days > 0 {
            let remaining_hours = hours % (24 * days);
            let remaining_minutes = minutes % (60 * hours);
            return Ok(format!("{days}d {remaining_hours}h {remaining_minutes}m"));
        }

        if hours > 0 {
            let remaining_minutes = minutes % (60 * hours);
            return Ok(format!("{hours}h {remaining_minutes}m"));
        }

        Ok(format!("{minutes}m"))
    }
}
