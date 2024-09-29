//! Module responsible for plotting/charts

use plotly::{Layout, Plot};

use crate::trace::TraceType;

/// Creates a plot in the `plots/` folder
///
/// Then opens it in the browser
pub fn single(trace: (TraceType, String)) {
    let title = trace.1;
    let mut plot = Plot::new();
    plot.add_trace(trace.0.get_inner());

    // sets the title of the plot
    let layout = Layout::new().title(format!("<b>{title}</b>"));
    plot.set_layout(layout);

    write_and_open_plot(&plot, &title);
}

/// Compares two traces in a single plot in the `plots/` folder
///
/// Then opens it in the browser
pub fn compare(trace_one: (TraceType, String), trace_two: (TraceType, String)) {
    let title = format!("{} vs {}", trace_one.1, trace_two.1);
    let mut plot = Plot::new();
    plot.add_trace(trace_one.0.get_inner());
    plot.add_trace(trace_two.0.get_inner());

    // sets the title of the plot
    let layout = Layout::new().title(format!("<b>{title}</b>"));
    plot.set_layout(layout);

    write_and_open_plot(&plot, &title);
}

/// Plots multiple traces in a single plot in the `plots/` folder
///
/// Then opens it in the browser
pub fn multiple(traces: Vec<TraceType>, title: &str) {
    let mut plot = Plot::new();

    for trace in traces {
        plot.add_trace(trace.get_inner());
    }

    // sets the title of the plot
    let layout = Layout::new().title(format!("<b>{title}</b>"));
    plot.set_layout(layout);

    write_and_open_plot(&plot, title);
}

/// Creates the plot .html in the plots/ folder and opens it in the browser
fn write_and_open_plot(plot: &Plot, title: &str) {
    // creates plots/ folder
    std::fs::create_dir_all("plots").unwrap();

    let title = crate::normalize_path(title);

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
