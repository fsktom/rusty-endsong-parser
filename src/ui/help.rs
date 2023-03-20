use crate::types::Color;

use unicode_width::UnicodeWidthStr;

/// Used by [`match_input()`][`super::match_input()`] for `help` command
///
/// Prints the available commands to the [`std::io::stdout`]
pub fn help() {
    // each entry: ["command", "alias", "description"]

    // META COMMANDS
    print("meta", &meta_commands());

    // PRINT COMMANDS
    print("print", &print_commands());

    print("print top", &print_top_commands());

    // GRAPH COMMANDS
    print("graph", &graph_commands());
}

/// Prints the commands
fn print(title: &str, commands: &Vec<[&str; 3]>) {
    println!(
        // "{title} commands" has to be centered to "=>"
        // so columns 22 and 23
        // everything before that has to be filled with '='
        // everything after that has to be filled with '=' till column 50
        "{}{}{}",
        Color::LightGreen,
        center_phrase(title, 22, 50),
        Color::Reset
    );
    for command in commands {
        println!(
            "{}{}{} => {}\n{}{}{}{}",
            Color::Red,
            adjust_length(command[0], 20),
            Color::Reset,
            // 20 (see above) - 4 (see below) ????
            prepend_spaces_for_newline(command[2], 16),
            Color::Pink,
            // 20 see above, 4 length of " => ", 7 length of "alias: "
            adjust_length("alias: ", 20 + 4 + 7),
            command[1],
            Color::Reset
        );
    }
}

/// Gives a `phrase` an appropriate amount of preceding spaces so it's `new_length` long
///
/// DOESN'T work correctly if `phrase` has linebreaks (`\n`)!
fn adjust_length(phrase: &str, new_length: usize) -> String {
    let ph = String::from(phrase);
    if UnicodeWidthStr::width(phrase) >= new_length {
        return ph;
    }

    // width_cjk bc Chinese/Japanese/Korean artist/album/song names
    let missing_spaces = new_length - UnicodeWidthStr::width_cjk(phrase);
    let mut spaces = String::with_capacity(missing_spaces);
    for _ in 0..missing_spaces {
        spaces.push(' ');
    }

    spaces + phrase
}

/// Prepends `num` spaces to secondary lines of `phrase`
///
/// First line is left as-is, but the following lines have `num` spaces
/// preceding them
fn prepend_spaces_for_newline(phrase: &str, num: usize) -> String {
    let mut new_phrase = String::new();
    // leave first line as-is
    // prepend spaces to other lines
    new_phrase.push_str(phrase.lines().next().unwrap());
    for line in phrase.lines().skip(1) {
        let mut spaces = String::with_capacity(num);
        for _ in 0..num {
            spaces.push(' ');
        }
        let temp = format!("\n{spaces}{line}");
        new_phrase.push_str(temp.as_str());
    }

    new_phrase
}

/// Centers "`phrase` commands" around columns start and start+1
fn center_phrase(phrase: &str, start: usize, end: usize) -> String {
    // let mut new_phrase = String::with_capacity(end);
    let mut new_phrase = format!(" {phrase} commands ");
    loop {
        let length = UnicodeWidthStr::width_cjk(new_phrase.as_str()) / 2 - 3;

        if length == start || length + 1 == start || length > end {
            break;
        }

        new_phrase = format!("={new_phrase}=");
    }

    while UnicodeWidthStr::width_cjk(new_phrase.as_str()) < end {
        new_phrase.push('=');
    }

    new_phrase
}

/// Returns meta commands
fn meta_commands<'a>() -> Vec<[&'a str; 3]> {
    vec![
        ["help", "h", "prints this command list"],
        ["exit", "quit", "exits the program"],
    ]
}

/// Returns print commands
fn print_commands<'a>() -> Vec<[&'a str; 3]> {
    vec![
        [
            "print artist",
            "part",
            "prints every album from the artist
        opens another prompt where you input the artist name",
        ],
        [
            "print album",
            "palb",
            "prints every song from the album
        opens another prompt where you input the artist name
        and then the album name",
        ],
        [
            "print song",
            "pson",
            "prints a song
        opens another prompt where you input the artist name
        and then the album name
        and then the song name",
        ],
        [
            "print songs",
            "psons",
            "prints a song with all the albums it may be from
        opens another prompt where you input the artist name
        and then the song name",
        ],
        [
            "print artist date",
            "partd",
            "prints every album from the artist within a date range
        opens another prompt where you input the artist name
        and then the date range",
        ],
        [
            "print album date",
            "palbd",
            "prints every song from the album within a date range
        opens another prompt where you input the artist name
        and then the album name",
        ],
        [
            "print song date",
            "psond",
            "prints a song within a date range
        opens another prompt where you input the artist name
        and then the album name
        and then the song name
        and then the date range",
        ],
        [
            "print songs date",
            "psonsd",
            "prints a song with all the albums it may be from within a date range
        opens another prompt where you input the artist name
        and then the song name
        and then the date range",
        ],
    ]
}

/// Returns print top commands
fn print_top_commands<'a>() -> Vec<[&'a str; 3]> {
    vec![
        ["print top artists", "ptarts", "prints top n artists"],
        ["print top albums", "ptalbs", "prints top n albums"],
        ["print top songs", "ptsons", "prints top n songs"],
    ]
}

/// Returns graph commands
fn graph_commands<'a>() -> Vec<[&'a str; 3]> {
    vec![["graph placeholder", "gphd", "placeholder description"]]
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
        // adjust_length() doesn't work properly with newlines (yet)
        assert_ne!(adjust_length("t\nt", 2), " t\n t");

        assert_eq!(prepend_spaces_for_newline("test", 20), "test");
        assert_eq!(
            prepend_spaces_for_newline("test\nsecond\nthird", 5),
            "test\n     second\n     third"
        );
    }
}
