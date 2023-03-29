use crate::display::date;
use crate::types::{Artist, SongEntries, SongEntry};
use crate::ui::user_input_date_parser;

use chrono::DateTime;
use chrono_tz::Tz;
use plotly::{Layout, Plot, Scatter};

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

    let mut plot = Plot::new();
    // TODO: make it display actual dates instead of UNIX timestamps xd
    plot.add_trace(Scatter::new(times, plays).name(art.name.as_str()));

    // sets the title of the plot the artist name
    let layout = Layout::new().title(format!("<b>{art}</b>").as_str().into());
    plot.set_layout(layout);

    // creates plots/ folder
    std::fs::create_dir_all("plots").unwrap();

    // opens the plot in the browser
    match std::env::consts::OS {
        // see https://github.com/igiagkiozis/plotly/issues/132#issuecomment-1488920563
        "windows" => {
            let path = format!(
                "{}\\plots\\{}.html",
                std::env::current_dir().unwrap().display(),
                &art.name
            );
            plot.write_html(path.as_str());
            std::process::Command::new("explorer")
                .arg(&path)
                .output()
                .unwrap();
        }
        "macos" => {
            let path = format!(
                "{}/plots/{}.html",
                std::env::current_dir().unwrap().display(),
                &art.name
            );
            plot.write_html(path.as_str());
            std::process::Command::new("open")
                .arg(&path)
                .output()
                .unwrap();
        }
        _ => {
            let path = format!(
                "{}/plots/{}.html",
                std::env::current_dir().unwrap().display(),
                &art.name
            );
            plot.write_html(path.as_str());

            // https://doc.rust-lang.org/book/ch12-05-working-with-environment-variables.html
            match std::env::var("BROWSER") {
                Ok(browser) => {
                    std::process::Command::new(&browser)
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

/// Used by [`artist()`] to get the dates of all of its occurrence
fn find_artist_dates(entries: &Vec<SongEntry>, art: &Artist) -> Vec<DateTime<Tz>> {
    let mut dates = Vec::<DateTime<Tz>>::new();

    for entry in entries {
        if entry.artist.eq(&art.name) {
            dates.push(entry.timestamp);
        }
    }

    dates.push(user_input_date_parser("now").unwrap());

    dates
}
