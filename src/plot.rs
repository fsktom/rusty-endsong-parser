//! Module responsible for plotting/charts
use plotly::{Layout, Plot, Scatter};

/// Responsible for plotting absolute plots
pub mod absolute;

/// Creates a plot in a `plots/` folder
///
/// Then opens it in the browser
fn create_plot(dates: Vec<i64>, plays: Vec<usize>, title: &str) {
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
