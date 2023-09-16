//! Module containing stuff for the `help` command

use unicode_width::UnicodeWidthStr;

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
    /// Base
    const BASE_LENGTH: usize = 20;

    /// Length of " => "
    const ARROW_LENGTH: usize = " => ".len();

    /// Length of "alias "
    const ALIAS_LENGTH: usize = "alias: ".len();

    // "{title} commands" has to be centered to "=>"
    // so columns 22 and 23
    // everything before that has to be filled with '='
    // everything after that has to be filled with '=' till column 50
    let centered_title = center_phrase(title, 22, 50);

    let reset = Color::Reset;
    let (light_green, red, pink) = (Color::LightGreen, Color::Red, Color::Pink);

    println!("{light_green}{centered_title}{reset}");

    let alias_pre = adjust_length("alias: ", BASE_LENGTH + ARROW_LENGTH + ALIAS_LENGTH);
    for Command(command, alias, description) in commands {
        println!(
            "{red}{}{reset} => {}\n{pink}{alias_pre}{alias}{reset}",
            adjust_length(command, BASE_LENGTH),
            // 20 (see above) - 4 (see below) ???? - 4 (implied in `description`)
            prepend_spaces_for_newline(description, BASE_LENGTH - ARROW_LENGTH - 4),
        );
    }
}

/// Gives a `phrase` an appropriate amount of preceding spaces so
/// that it is at least `new_length` long
///
/// DOESN'T work correctly if `phrase` has linebreaks (`'\n'`)!
/// Use [`prepend_spaces_for_newline()`] for the following
/// lines instead
///
/// # Examples
/// ```ignore
/// assert_eq!(
///     adjust_length("print top artists", 20),
///     "   print top artists"
/// );
/// ```
fn adjust_length(phrase: &str, new_length: usize) -> String {
    let current_length = phrase.width();
    if current_length >= new_length {
        return String::from(phrase);
    }

    let missing_spaces = new_length - current_length;
    let spaces = " ".repeat(missing_spaces);

    spaces + phrase
}

/// Prepends `num` spaces to secondary lines of `phrase`
///
/// First line is left as-is, but the following lines have `num` spaces
/// preceding them
fn prepend_spaces_for_newline(phrase: &str, num: usize) -> String {
    let mut lines = phrase.lines();

    // leave first line as-is
    let mut new_phrase = String::from(lines.next().unwrap());

    // prepend spaces to other lines
    let spaces = " ".repeat(num);
    for line in lines {
        let new_line = format!("\n{spaces}{line}");
        new_phrase.push_str(&new_line);
    }

    new_phrase
}

/// Centers "`phrase` commands" around columns `start` and `start`+1
/// by filling it with '=' on both sides (not evenly)
///
/// start and start+1 are positions of "=>"
///
/// end is column where the final '=' should be put
fn center_phrase(phrase: &str, start: usize, end: usize) -> String {
    let mut new_phrase = format!(" {phrase} commands ");
    loop {
        // not really sure if this actually centers it, but it's close enough, right? :(
        let length = new_phrase.width() / 2 - 3;

        if length == start || length + 1 == start || length > end {
            break;
        }

        new_phrase = format!("={new_phrase}=");
    }

    let missing_symbols = "=".repeat(end - new_phrase.width());

    new_phrase + &missing_symbols
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
            "prints the time spent listening in a specific date range
            opens another prompt where you input the date range",
        ),
        Command(
            "print max time",
            "pmt",
            "calculates the dates during which you listened
            the most to music for a given duration",
        ),
        Command(
            "print artist",
            "part",
            "prints every album from the artist
            opens another prompt where you input the artist name",
        ),
        Command(
            "print album",
            "palb",
            "prints every song from the album
            opens another prompt where you input the artist name
            and then the album name",
        ),
        Command(
            "print song",
            "pson",
            "prints a song
            opens another prompt where you input the artist name
            and then the album name
            and then the song name",
        ),
        Command(
            "print songs",
            "psons",
            "prints a song with all the albums it may be from
            opens another prompt where you input the artist name
            and then the song name",
        ),
        Command(
            "print artist date",
            "partd",
            "prints every album from the artist within a date range
            opens another prompt where you input the artist name
            and then the date range",
        ),
        Command(
            "print album date",
            "palbd",
            "prints every song from the album within a date range
            opens another prompt where you input the artist name
            and then the album name",
        ),
        Command(
            "print song date",
            "psond",
            "prints a song within a date range
            opens another prompt where you input the artist name
            and then the album name
            and then the song name
            and then the date range",
        ),
        Command(
            "print songs date",
            "psonsd",
            "prints a song with all the albums it may be from within a date range
            opens another prompt where you input the artist name
            and then the song name
            and then the date range",
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
            "creates a plot of the absolute amount of plays of the given aspect
            and opens it in the web browser",
        ),
        Command(
            "plot rel",
            "gr",
            "creates a plot of the amount of plays of the given aspect relative
            to all, the artist or album
            and opens it in the web browser",
        ),
        Command(
            "plot compare",
            "gc",
            "creates a plot of two traces - see `plot`
            and opens it in the web browser",
        ),
        Command(
            "plot compare rel",
            "gcr",
            "creates a plot of two relative traces - see `plot rel`
            and opens it in the web browser",
        ),
        Command(
            "plot top",
            "gt",
            "creates a plot of the absolute traces of top n aspects
            and opens it in the web browser",
        ),
        Command(
            "plot artist albums",
            "gaa",
            "creates a plot of the absolute traces of all albums
            of the given artist
            and opens it in the web browser",
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whitespace() {
        assert_eq!(
            adjust_length("print top artists", 20),
            "   print top artists"
        );
        assert_eq!(adjust_length("print top artists", 0), "print top artists");
        // adjust_length() doesn't work properly with newlines (yet)
        assert_ne!(adjust_length("t\nt", 2), " t\n t");

        assert_eq!(prepend_spaces_for_newline("test", 20), "test");
        assert_eq!(
            prepend_spaces_for_newline("test\nsecond\nthird", 5),
            "test\n     second\n     third"
        );
    }
}
