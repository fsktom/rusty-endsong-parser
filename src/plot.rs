//! Module responsible for plotting/charts
use crate::types::{Music, SongEntry};
use crate::ui::user_input_date_parser;

use chrono::DateTime;
use chrono_tz::Tz;
use plotly::{Layout, Plot, Scatter};

/// Responsible for plotting absolute plots
pub mod absolute;

/// Responsible for plotting plots relative to sum of plays
pub mod relative;

/// Creates a plot in a `plots/` folder
///
/// Then opens it in the browser
fn create_plot<Y>(dates: Vec<i64>, plays: Vec<Y>, title: &str)
where
    Y: serde::Serialize + Clone + 'static,
    // see https://github.com/igiagkiozis/plotly/blob/8903ff03ce9e8183624c40ccf7ddf863799cb92e/plotly/src/traces/scatter.rs#L292-L303
{
    let mut plot = Plot::new();
    // TODO: make it display actual dates instead of UNIX timestamps xd
    plot.add_trace(Scatter::new(dates, plays).name(title));

    // sets the title of the plot
    let layout = Layout::new().title(format!("<b>{title}</b>").as_str().into());
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

/// Returns the dates of all occurrences of the `aspect`
///
/// * `add_now` - with this set to true, it will put the current time as the last date,
/// otherwise it will be the last occurrence of `aspect`
pub fn find_dates<Asp: Music>(
    entries: &Vec<SongEntry>,
    aspect: &Asp,
    add_now: bool,
) -> Vec<DateTime<Tz>> {
    let mut dates = Vec::<DateTime<Tz>>::new();

    for entry in entries {
        if aspect.is_entry(entry) {
            dates.push(entry.timestamp);
        }
    }

    dates.sort();

    if add_now {
        dates.push(user_input_date_parser("now").unwrap());
    }
    dates
}
