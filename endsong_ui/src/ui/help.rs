//! Module containing stuff for the `help` command

use super::Color;

/// Represents a command
///
/// Fields are name, alias, description
#[derive(Copy, Clone, Debug)]
struct Command(&'static str, &'static str, &'static str);

/// Used by [`match_input()`][`super::match_input()`] for `help` command
///
/// Prints the available commands to the [`std::io::stdout`]
pub fn help() {
    // each entry: ["command", "alias", "description"]

    // META COMMANDS
    print("meta", meta_commands());

    // PRINT COMMANDS
    print("print", print_commands());
    print("print top", print_top_commands());

    // GRAPH COMMANDS
    print("graph/plot", plot_commands());
}

/// Prints the commands
fn print(title: &str, commands: &[Command]) {
    /// Length of command name
    const COMMAND_LENGTH: usize = 20;

    /// Used to separate command and its description
    const ARROW: &str = " => ";

    /// Length of " => "
    const ARROW_LENGTH: usize = ARROW.len();

    /// Length of one command description line
    const DESCRIPTION_LENGTH: usize = 2 * COMMAND_LENGTH;

    /// Length of whole line
    const FULL_LENGTH: usize = COMMAND_LENGTH + ARROW_LENGTH + DESCRIPTION_LENGTH;

    /// Prefixes the actual command alias
    const ALIAS: &str = "alias: ";

    /// Spaces going to start of description
    const INDENT: &str = crate::spaces(COMMAND_LENGTH + ARROW_LENGTH);

    let phrase = format!(" {title} commands ");
    // centered, filled with '=' on both sides
    // see https://doc.rust-lang.org/std/fmt/#fillalignment
    let centered_title = format!("{phrase:=^FULL_LENGTH$}");

    let reset = Color::Reset;
    let (light_green, red, pink) = (Color::LightGreen, Color::Red, Color::Pink);

    println!("{light_green}{centered_title}{reset}");

    for Command(command, alias, description) in commands {
        let description_lines = textwrap::wrap(description, DESCRIPTION_LENGTH);
        let description_first = description_lines.first().unwrap();

        println!("{red}{command:>COMMAND_LENGTH$}{reset}{ARROW}{description_first}");
        for line in description_lines.iter().skip(1) {
            println!("{INDENT}{line}",);
        }
        println!("{pink}{INDENT}{ALIAS}{alias}{reset}");
    }
}

/// Returns meta commands
const fn meta_commands() -> &'static [Command] {
    &[
        Command("help", "h", "prints this command list"),
        Command("exit", "quit", "exits the program"),
    ]
}

/// Returns print commands
const fn print_commands() -> &'static [Command] {
    &[
        Command("print time", "pt", "prints the total time spent listening"),
        Command(
            "print time date",
            "ptd",
            "prints the time spent listening in a specific date range",
        ),
        Command(
            "print max time",
            "pmt",
            "calculates the dates during which you listened the most to music for a given duration",
        ),
        Command(
            "print artist",
            "part",
            "prints every album from the given artist",
        ),
        Command(
            "print album",
            "palb",
            "prints every song from the given album",
        ),
        Command("print song", "pson", "prints a song's stats"),
        Command(
            "print songs",
            "psons",
            "prints a song with all the albums it may be from",
        ),
        Command(
            "print artist date",
            "partd",
            "prints every album from the artist within a date range",
        ),
        Command(
            "print album date",
            "palbd",
            "prints every song from the album within a date range",
        ),
        Command(
            "print song date",
            "psond",
            "prints a song's stats within a date range",
        ),
        Command(
            "print songs date",
            "psonsd",
            "prints a song with all the albums it may be from within a date range",
        ),
    ]
}

/// Returns print top commands
const fn print_top_commands() -> &'static [Command] {
    &[
        Command("print top artists", "ptarts", "prints top n artists"),
        Command("print top albums", "ptalbs", "prints top n albums"),
        Command("print top songs", "ptsons", "prints top n songs"),
    ]
}

/// Returns graph commands
const fn plot_commands() -> &'static [Command] {
    &[
        Command(
            "plot",
            "g",
            "creates a plot of the absolute amount of plays of the given aspect and opens it in the web browser",
        ),
        Command(
            "plot rel",
            "gr",
            "creates a plot of the amount of plays of the given aspect relative to all, the artist or album and opens it in the web browser",
        ),
        Command(
            "plot compare",
            "gc",
            "creates a plot of two traces (see `plot`) and opens it in the web browser",
        ),
        Command(
            "plot compare rel",
            "gcr",
            "creates a plot of two relative traces (see `plot rel`) and opens it in the web browser",
        ),
        Command(
            "plot top",
            "gt",
            "creates a plot of the absolute traces of top n aspects and opens it in the web browser",
        ),
        Command(
            "plot artist albums",
            "gaa",
            "creates a plot of the absolute traces of all albums of the given artist and opens it in the web browser",
        ),
    ]
}
