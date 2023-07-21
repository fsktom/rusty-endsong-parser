//! Module responsible for plotting/charts

pub mod absolute;
pub mod relative;

use chrono::{DateTime, TimeZone};
use chrono_tz::Tz;
use plotly::{Layout, Plot, Trace};

use crate::types::{Music, SongEntry};

/// Creates a plot in the `plots/` folder
///
/// Then opens it in the browser
pub fn single(trace: (Box<dyn Trace>, String)) {
    let title = trace.1;
    let mut plot = Plot::new();
    plot.add_trace(trace.0);

    // sets the title of the plot
    let layout = Layout::new().title(format!("<b>{title}</b>").as_str().into());
    plot.set_layout(layout);

    write_and_open_plot(&plot, title);
}

/// Compares two traces in a single plot in the `plots/` folder
///
/// Then opens it in the browser
pub fn compare(trace_one: (Box<dyn Trace>, String), trace_two: (Box<dyn Trace>, String)) {
    let title = format!("{} vs {}", trace_one.1, trace_two.1);
    let mut plot = Plot::new();
    plot.add_trace(trace_one.0);
    plot.add_trace(trace_two.0);

    // sets the title of the plot
    let layout = Layout::new().title(format!("<b>{title}</b>").as_str().into());
    plot.set_layout(layout);

    write_and_open_plot(&plot, title);
}

/// Returns the dates of all occurrences of the `aspect`
///
/// * `add_now` - with this set to true, it will put the current time as the last date,
/// otherwise it will be the last occurrence of `aspect`
pub fn find_dates<Asp: Music>(
    entries: &[SongEntry],
    aspect: &Asp,
    add_now: bool,
) -> Vec<DateTime<Tz>> {
    let mut dates = Vec::<DateTime<Tz>>::new();

    for entry in entries {
        if aspect.is_entry(entry) {
            dates.push(entry.timestamp);
        }
    }

    if add_now {
        dates.push(
            crate::LOCATION_TZ
                .timestamp_millis_opt(chrono::offset::Local::now().timestamp_millis())
                .unwrap(),
        );
    }

    // should be sorted because &[SongEntries] should have been sorted at the beginning
    dates
}

/// Creates the plot .html in the plots/ folder and opens it in the browser
fn write_and_open_plot(plot: &Plot, mut title: String) {
    // creates plots/ folder
    std::fs::create_dir_all("plots").unwrap();

    title = normalize_path(title.as_str());

    // opens the plot in the browser
    match std::env::consts::OS {
        // see https://github.com/igiagkiozis/plotly/issues/132#issuecomment-1488920563
        "windows" => {
            let path = format!(
                "{}\\plots\\{}.html",
                std::env::current_dir().unwrap().display(),
                title
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
                title
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
                title
            );
            plot.write_html(path.as_str());

            // https://doc.rust-lang.org/book/ch12-05-working-with-environment-variables.html
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

/// Replaces Windows forbidden symbols in path with a '_'
fn normalize_path(path: &str) -> String {
    // https://stackoverflow.com/a/31976060
    let forbidden_characters = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    let mut new_path = String::with_capacity(path.len());

    for ch in path.chars() {
        if forbidden_characters.contains(&ch) {
            // replace a forbidden symbol with an underscore (for now...)
            new_path.push('_');
        } else {
            new_path.push(ch);
        }
    }

    new_path
}
