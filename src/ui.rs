//! Module responsible for handling the CLI

use crate::types::{Aspect, AspectFull, Color, NotFoundError, SongEntries};
use crate::LOCATION_TZ;

use std::borrow::Cow;
use std::error::Error;

use chrono::{DateTime, TimeZone};
use chrono_tz::Tz;
use rustyline::{
    error::ReadlineError, highlight::Highlighter, history::FileHistory, ColorMode, Config, Editor,
};
use rustyline::{Completer, Helper, Hinter, Validator};
use unicode_width::UnicodeWidthStr;

/// Prompt used for top-level shell commands
///
/// green `>>>` with [`ShellHelper`]
const PROMPT_COMMAND: &str = ">>> ";

/// Prompt used for main arguments like artist, album and song name
///
/// cyan `  >>` with [`ShellHelper`]
const PROMPT_MAIN: &str = "  >> ";

/// Prompt used for additional arguments like the date range
///
/// red `   >` with [`ShellHelper`]
const PROMPT_SECONDARY: &str = "   > ";

/// Helper for [`Editor`]
#[derive(Completer, Helper, Hinter, Validator)]
struct ShellHelper;
impl Highlighter for ShellHelper {
    // makes the prompt in rl.readline() change color depending on the prompt
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _default: bool,
    ) -> std::borrow::Cow<'b, str> {
        match prompt {
            PROMPT_COMMAND => Cow::Owned(format!(
                "{}{}{}",
                Color::Green,
                PROMPT_COMMAND,
                Color::Reset
            )),
            PROMPT_MAIN => Cow::Owned(format!("{}{}{}", Color::Cyan, PROMPT_MAIN, Color::Reset)),
            PROMPT_SECONDARY => Cow::Owned(format!(
                "{}{}{}",
                Color::Red,
                PROMPT_SECONDARY,
                Color::Reset
            )),
            _ => Cow::Borrowed(prompt),
        }
    }
}

/// Starts the CLI/shell instance
pub fn start(entries: &SongEntries) {
    println!("=== INTERACTIVE MODE ACTIVATED ===");
    println!("PRESS 'CTRL+C' TO EXIT THE PROGRAM");
    println!("TYPE 'help' FOR AVAILABLE COMMANDS");

    // inspired by
    // https://github.com/trane-project/trane-cli/blob/master/src/main.rs
    let config = Config::builder()
        .auto_add_history(true)
        .color_mode(ColorMode::Enabled)
        .history_ignore_space(true)
        .build();

    let mut rl = Editor::<ShellHelper, FileHistory>::with_config(config)
        .expect("Sorry, there's been an error!");

    let helper = ShellHelper;
    rl.set_helper(Some(helper));

    let history_path = std::path::Path::new(".rep_history");
    if !history_path.exists() {
        match std::fs::File::create(history_path) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to create history file: {e}");
            }
        }
    }
    match rl.load_history(history_path) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Failed to load history file at .rep_history: {e}");
        }
    }

    loop {
        let line = rl.readline(PROMPT_COMMAND);
        match line {
            Ok(usr_input) => {
                if &usr_input == "exit" || &usr_input == "quit" {
                    break;
                }
                match_input(&usr_input, entries, &mut rl).unwrap_or_else(|e| handle_error(&e));
            }
            Err(ReadlineError::Interrupted) => {
                eprintln!("Ctrl+C - execution has stopped!");
                break;
            }
            Err(ReadlineError::Eof) => {
                eprintln!("CTRL-D - execution has stopped!");
                break;
            }
            Err(err) => {
                eprintln!("Execution has stopped! - Error: {err}");
                break;
            }
        }
    }

    match rl.save_history(history_path) {
        Ok(_) => (),
        Err(e) => {
            eprintln!(
                "Failed to save history to file {}: {e}",
                &history_path.to_str().unwrap()
            );
        }
    }
}

/// Handles errors thrown by [`match_input()`] in [`start()`]
///
/// Prints error messages for [`NotFoundError`],
/// [`ParseError`][`chrono::format::ParseError`],
/// and [`ParseIntError`][`std::num::ParseIntError`]
#[allow(clippy::borrowed_box)]
fn handle_error(err: &Box<dyn Error>) {
    // https://users.rust-lang.org/t/matching-errorkind-from-boxed-error/30667/3
    // also thx ChatGPT
    match err.as_ref() {
        not_found if not_found.is::<NotFoundError>() => eprintln!("{not_found}"),
        date if date.is::<chrono::format::ParseError>() => {
            eprintln!("Invalid date! Make sure you input the date in YYYY-MM-DD format.");
        }
        num_parse if num_parse.is::<std::num::ParseIntError>() => eprintln!("Incorrect number!"),
        // e.g. if user presses CTRl+C/+D/.. in a main or secondary prompt,
        // it should just stop the command and go back to command prompt
        read_error if read_error.is::<ReadlineError>() => (),
        _ => eprintln!("An error has occured! - {err}",),
    }
}

/// Decides what to do with user input
fn match_input(
    inp: &str,
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(), Box<dyn Error>> {
    match inp {
        "help" | "h" => help(),
        "print artist" | "part" => match_print_artist(entries, rl)?,
        "print album" | "palb" => match_print_album(entries, rl)?,
        "print song" | "pson" => match_print_song(entries, rl)?,
        "print songs" | "psons" => match_print_songs(entries, rl)?,
        "print artist date" | "partd" => match_print_artist_date(entries, rl)?,
        "print album date" | "palbd" => match_print_album_date(entries, rl)?,
        "print song date" | "psond" => match_print_song_date(entries, rl)?,
        "print songs date" | "psonsd" => match_print_songs_date(entries, rl)?,
        "print top artists" | "ptarts" => match_print_top(entries, rl, &Aspect::Artists)?,
        "print top albums" | "ptalbs" => match_print_top(entries, rl, &Aspect::Albums)?,
        "print top songs" | "ptsons" => match_print_top(entries, rl, &Aspect::Songs)?,
        // when you press ENTER -> nothing happens, new prompt
        "" => (),
        _ => {
            println!(
                "Command not found! Type {}help{} to print available commands",
                Color::Red,
                Color::Reset
            );
        }
    }
    Ok(())
}

/// Used by [`match_input()`] for `print artist` command
fn match_print_artist(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(), Box<dyn Error>> {
    // prompt: artist name
    println!("Artist name?");
    let usr_input_art = rl.readline(PROMPT_MAIN)?;
    let art = entries.find().artist(&usr_input_art)?;

    entries.print_aspect(&AspectFull::Artist(&art));
    Ok(())
}

/// Used by [`match_input()`] for `print artist date` command
///
/// Basically [`match_print_artist()`] but with date functionality
fn match_print_artist_date(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(), Box<dyn Error>> {
    // 1st prompt: artist name
    println!("Artist name?");
    let usr_input_art = rl.readline(PROMPT_MAIN)?;
    let art = entries.find().artist(&usr_input_art)?;

    // 2nd prompt: start date
    println!("Start date? YYYY-MM-DD or 'start'");
    let usr_input_start_date = rl.readline(PROMPT_SECONDARY)?;
    let start_date = user_input_date_parser(&usr_input_start_date)?;

    // 3rd prompt: end date
    println!("End date? YYYY-MM-DD or 'now'");
    let usr_input_end_date = rl.readline(PROMPT_SECONDARY)?;
    let end_date = user_input_date_parser(&usr_input_end_date)?;

    entries.print_aspect_date(&AspectFull::Artist(&art), &start_date, &end_date);
    Ok(())
}

/// Used by [`match_input()`] for `print album` command
fn match_print_album(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(), Box<dyn Error>> {
    // 1st prompt: artist name
    println!("Artist name?");
    let usr_input_art = rl.readline(PROMPT_MAIN)?;
    let art = entries.find().artist(&usr_input_art)?;

    // 2nd prompt: album name
    println!("Album name?");
    let usr_input_alb = rl.readline(PROMPT_MAIN)?;
    let alb = entries.find().album(&usr_input_alb, &art.name)?;

    entries.print_aspect(&AspectFull::Album(&alb));
    Ok(())
}

/// Used by [`match_input()`] for `print album date` command
///
/// Basically [`match_print_album()`] but with date functionality
fn match_print_album_date(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(), Box<dyn Error>> {
    // 1st prompt: artist name
    println!("Artist name?");
    let usr_input_art = rl.readline(PROMPT_MAIN)?;
    let art = entries.find().artist(&usr_input_art)?;

    // 2nd prompt: album name
    println!("Album name?");
    let usr_input_alb = rl.readline(PROMPT_MAIN)?;
    let alb = entries.find().album(&usr_input_alb, &art.name)?;

    // 3rd prompt: start date
    println!("Start date? YYYY-MM-DD or 'start'");
    let usr_input_start_date = rl.readline(PROMPT_SECONDARY)?;
    let start_date = user_input_date_parser(&usr_input_start_date)?;

    // 4th prompt: end date
    println!("End date? YYYY-MM-DD or 'now'");
    let usr_input_end_date = rl.readline(PROMPT_SECONDARY)?;
    let end_date = user_input_date_parser(&usr_input_end_date)?;

    entries.print_aspect_date(&AspectFull::Album(&alb), &start_date, &end_date);
    Ok(())
}

/// Used by [`match_input()`] for `print song` command
fn match_print_song(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(), Box<dyn Error>> {
    // 1st prompt: artist name
    println!("Artist name?");
    let usr_input_art = rl.readline(PROMPT_MAIN)?;
    let art = entries.find().artist(&usr_input_art)?;

    // 2nd prompt: album name
    println!("Album name?");
    let usr_input_alb = rl.readline(PROMPT_MAIN)?;
    let alb = entries.find().album(&usr_input_alb, &art.name)?;

    // 3rd prompt: song name
    println!("Song name?");
    let usr_input_son = rl.readline(PROMPT_MAIN)?;
    let son = entries
        .find()
        .song_from_album(&usr_input_son, &alb.name, &alb.artist.name)?;

    entries.print_aspect(&AspectFull::Song(&son));
    Ok(())
}

/// Used by [`match_input()`] for `print song date` command
///
/// Basically [`match_print_song()`] but with date functionality
fn match_print_song_date(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(), Box<dyn Error>> {
    // 1st prompt: artist name
    println!("Artist name?");
    let usr_input_art = rl.readline(PROMPT_MAIN)?;
    let art = entries.find().artist(&usr_input_art)?;

    // 2nd prompt: album name
    println!("Album name?");
    let usr_input_alb = rl.readline(PROMPT_MAIN)?;
    let alb = entries.find().album(&usr_input_alb, &art.name)?;

    // 3rd prompt: song name
    println!("Song name?");
    let usr_input_son = rl.readline(PROMPT_MAIN)?;
    let son = entries
        .find()
        .song_from_album(&usr_input_son, &alb.name, &alb.artist.name)?;

    // 4th prompt: start date
    println!("Start date? YYYY-MM-DD or 'start'");
    let usr_input_start_date = rl.readline(PROMPT_SECONDARY)?;
    let start_date = user_input_date_parser(&usr_input_start_date)?;

    // 5th prompt: end date
    println!("End date? YYYY-MM-DD or 'now'");
    let usr_input_end_date = rl.readline(PROMPT_SECONDARY)?;
    let end_date = user_input_date_parser(&usr_input_end_date)?;

    entries.print_aspect_date(&AspectFull::Song(&son), &start_date, &end_date);
    Ok(())
}

/// Used by [`match_input()`] for `print songs` command
fn match_print_songs(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(), Box<dyn Error>> {
    // 1st prompt: artist name
    println!("Artist name?");
    let usr_input_art = rl.readline(PROMPT_MAIN)?;
    let art = entries.find().artist(&usr_input_art)?;

    // 2nd prompt: song name
    println!("Song name?");
    let usr_input_son = rl.readline(PROMPT_MAIN)?;
    let songs = entries.find().song(&usr_input_son, &art.name)?;

    // if there are multiple songs with that name found
    if songs.len() > 1 {
        println!(
            "I've found {} songs named {} from {}!",
            songs.len(),
            &songs[0].name,
            &songs[0].album.artist.name
        );
    }
    for song in songs {
        entries.print_aspect(&AspectFull::Song(&song));
    }
    Ok(())
}

/// Used by [`match_input()`] for `print songs date` command
fn match_print_songs_date(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(), Box<dyn Error>> {
    // 1st prompt: artist name
    println!("Artist name?");
    let usr_input_art = rl.readline(PROMPT_MAIN)?;
    let art = entries.find().artist(&usr_input_art)?;

    // 2nd prompt: song name
    println!("Song name?");
    let usr_input_son = rl.readline(PROMPT_MAIN)?;
    let songs = entries.find().song(&usr_input_son, &art.name)?;

    // 3rd prompt: start date
    println!("Start date? YYYY-MM-DD or 'start'");
    let usr_input_start_date = rl.readline(PROMPT_SECONDARY)?;
    let start_date = user_input_date_parser(&usr_input_start_date)?;

    // 4th prompt: end date
    println!("End date? YYYY-MM-DD or 'now'");
    let usr_input_end_date = rl.readline(PROMPT_SECONDARY)?;
    let end_date = user_input_date_parser(&usr_input_end_date)?;

    // if there are multiple songs with that name found
    if songs.len() > 1 {
        println!(
            "I've found {} songs named {} from {}!",
            songs.len(),
            &songs[0].name,
            &songs[0].album.artist.name
        );
    }
    for song in songs {
        entries.print_aspect_date(&AspectFull::Song(&song), &start_date, &end_date);
    }
    Ok(())
}

/// Used by [`match_input()`] for `print top artists/albums/songs` commands
fn match_print_top(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
    asp: &Aspect,
) -> Result<(), Box<dyn Error>> {
    // prompt: top n
    println!("How many Top {asp}?");
    let usr_input_n = rl.readline(PROMPT_MAIN)?;
    let num: usize = usr_input_n.parse()?;

    entries.print_top(asp, num);
    Ok(())
}

/// Used by [`match_input()`] for `help` command
///
/// Prints the available commands to the [`std::io::stdout`]
#[allow(clippy::too_many_lines)]
#[allow(clippy::vec_init_then_push)]
fn help() {
    /// Prints the commands
    fn print(title: &str, commands: &Vec<[&str; 3]>) {
        println!(
            "{}=== {} commands ==={}",
            Color::LightGreen,
            title,
            Color::Reset
        );
        for command in commands {
            println!(
                "{}{}{} => {}\n{}{}{}{}",
                Color::Red,
                spaces(command[0], 20, true),
                Color::Reset,
                // 20 (see above) - 4 (see below) ????
                spaces_for_newline(command[2], 16),
                Color::Pink,
                // 20 see above, 4 length of " => ", 7 length of "alias: "
                spaces("alias: ", 20 + 4 + 7, true),
                command[1],
                Color::Reset
            );
        }
    }

    // each entry: ["command", "alias", "description"]

    // META COMMANDS
    let mut meta_commands: Vec<[&str; 3]> = Vec::new();
    meta_commands.push(["help", "h", "prints this command list"]);
    meta_commands.push(["exit", "quit", "exits the program"]);
    print("meta", &meta_commands);

    // PRINT COMMANDS
    let mut print_commands: Vec<[&str; 3]> = Vec::new();
    print_commands.push([
        "print artist",
        "part",
        "prints every album from the artist
        opens another prompt where you input the artist name",
    ]);
    print_commands.push([
        "print album",
        "palb",
        "prints every song from the album
        opens another prompt where you input the artist name
        and then the album name",
    ]);
    print_commands.push([
        "print song",
        "pson",
        "prints a song
        opens another prompt where you input the artist name
        and then the album name
        and then the song name",
    ]);
    print_commands.push([
        "print songs",
        "psons",
        "prints a song with all the albums it may be from
        opens another prompt where you input the artist name
        and then the song name",
    ]);

    print_commands.push([
        "print artist date",
        "partd",
        "prints every album from the artist within a date range
        opens another prompt where you input the artist name
        and then the date range",
    ]);
    print_commands.push([
        "print album date",
        "palbd",
        "prints every song from the album within a date range
        opens another prompt where you input the artist name
        and then the album name",
    ]);
    print_commands.push([
        "print song date",
        "psond",
        "prints a song within a date range
        opens another prompt where you input the artist name
        and then the album name
        and then the song name
        and then the date range",
    ]);
    print_commands.push([
        "print songs date",
        "psonsd",
        "prints a song with all the albums it may be from within a date range
        opens another prompt where you input the artist name
        and then the song name
        and then the date range",
    ]);
    print("print", &print_commands);

    let mut print_top_commands: Vec<[&str; 3]> = Vec::new();
    print_top_commands.push(["print top artists", "ptarts", "prints top n artists"]);
    print_top_commands.push(["print top albums", "ptalbs", "prints top n albums"]);
    print_top_commands.push(["print top songs", "ptsons", "prints top n songs"]);
    print("print top", &print_top_commands);

    // GRAPH COMMANDS
    let mut graph_commands: Vec<[&str; 3]> = Vec::new();
    graph_commands.push(["graph placeholder", "gphd", "placeholder description"]);
    print("graph", &graph_commands);
}

/// Gives a [`String`] an appropriate amount of leading spaces so it's `num` long
fn spaces(phrase: &str, num: usize, prepend: bool) -> String {
    let ph = String::from(phrase);
    if UnicodeWidthStr::width(phrase) >= num {
        return ph;
    }

    // width_cjk bc Chinese/Japanese/Korean artist/album/song names
    let missing_spaces = num - UnicodeWidthStr::width_cjk(phrase);
    let mut spaces = String::with_capacity(missing_spaces);
    for _ in 0..missing_spaces {
        spaces.push(' ');
    }

    if prepend {
        return spaces + phrase;
    }
    phrase.to_owned() + spaces.as_str()
}

/// todo()!
fn spaces_for_newline(phrase: &str, num: usize) -> String {
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

/// used by `*_date` functions in this module for when the user inputs a date
///
/// # Arguments
/// * `usr_input` - in YYYY-MM-DD format or 'now' or 'start'
fn user_input_date_parser(usr_input: &str) -> Result<DateTime<Tz>, chrono::format::ParseError> {
    let date_str = match usr_input {
        "now" => {
            return Ok(LOCATION_TZ
                .timestamp_millis_opt(chrono::offset::Local::now().timestamp_millis())
                .unwrap())
        }
        // TODO! not hardcode this lol -> actual earlierst entry in endsong
        "start" => String::from("1980-01-01T00:00:00Z"),
        // usr_input should be in YYYY-MM-DD format
        _ => format!("{usr_input}T00:00:00Z"),
    };

    // "%FT%TZ" is equivalent to "%Y-%m-%dT%H:%M:%SZ"
    // see <https://docs.rs/chrono/latest/chrono/format/strftime/index.html>
    LOCATION_TZ.datetime_from_str(&date_str, "%FT%TZ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_parser() {
        // correctly formatted input date
        assert_eq!(
            user_input_date_parser("2020-06-06").unwrap(),
            LOCATION_TZ
                .datetime_from_str("2020-06-06T00:00:00Z", "%Y-%m-%dT%H:%M:%SZ")
                .unwrap()
        );
        // https://users.rust-lang.org/t/idiomatic-way-of-testing-result-t-error/2171/4
        assert!(user_input_date_parser("2020-06-06").is_ok());

        // incorrectly formatted input date
        assert!(user_input_date_parser("feer3er3").is_err());
    }
}
