//! Module responsible for handling the CLI

use crate::types::{
    plot_compare, plot_single, Aspect, AspectFull, Color, NotFoundError, SongEntries, Trace,
};
use crate::LOCATION_TZ;

use std::borrow::Cow;
use std::error::Error;
use std::vec;

use chrono::{DateTime, TimeZone};
use chrono_tz::Tz;
use rustyline::{completion::Completer, Helper, Hinter, Validator};
use rustyline::{
    error::ReadlineError, highlight::Highlighter, history::FileHistory, ColorMode, Config, Editor,
};

/// Module containing stuff for the `help` command
mod help;

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

/// Errors raised by [`match_plot_album_relative()`] and
/// [`match_plot_song_relative`]
///
/// when user argument for relative to what is invalid
#[derive(Debug)]
enum InvalidArgumentError {
    /// Error message: Invalid argument! Try using 'all' or 'artist' next time
    Artist,
    /// Error message: Invalid argument! Try using 'all', 'artist' or 'album' next time
    Album,
    /// Error message: Invalid argument! Try using 'artist', 'album' or 'song' next time
    Aspect,
}
impl std::fmt::Display for InvalidArgumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvalidArgumentError::Artist => {
                write!(f, "Invalid argument! Try using 'all' or 'artist' next time")
            }
            InvalidArgumentError::Album => write!(
                f,
                "Invalid argument! Try using 'all', 'artist' or 'album' next time"
            ),
            InvalidArgumentError::Aspect => write!(
                f,
                "Invalid argument! Try using 'artist', 'album' or 'song' next time"
            ),
        }
    }
}
impl Error for InvalidArgumentError {}

/// Helper for [`Editor`]
#[derive(Helper, Hinter, Validator)]
struct ShellHelper {
    /// List containing all the possible completes for Tab
    completer_list: Vec<String>,
}
impl ShellHelper {
    /// Creates a new [`ShellHelper`]
    fn new() -> Self {
        Self {
            completer_list: vec![],
        }
    }

    /// Makes tab-complete list empty
    fn reset(&mut self) {
        self.completer_list = vec![];
    }

    /// Changes tab-complete to prompt commands
    fn complete_commands(&mut self) {
        let temp = [
            "help",
            "print time",
            "print time date",
            "print artist",
            "print album",
            "print song",
            "print songs",
            "print artist date",
            "print album date",
            "print song date",
            "print songs date",
            "print top artists",
            "print top songs",
            "plot",
            "plot rel",
            "plot compare",
            "plot compare rel",
        ];

        // so that I don't have to call .to_string() on every single entry in the array above...
        self.completer_list = temp.iter().map(|s| s.to_string()).collect::<Vec<String>>();
    }

    /// Changes tab-complete to ["artist", "album", "song"]
    fn complete_aspects(&mut self) {
        self.completer_list = vec![
            "artist".to_string(),
            "album".to_string(),
            "song".to_string(),
        ];
    }

    /// Changes tab-complete to all artists
    fn complete_list(&mut self, completer_list: Vec<String>) {
        self.completer_list = completer_list;
    }
}
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
impl Completer for ShellHelper {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let word = &line[0..pos];
        let mut possibilities = vec![];
        self.completer_list
            .iter()
            .filter(|compl| compl.starts_with(word))
            .for_each(|w| possibilities.push(w.to_string()));
        // assumes no escape characters...
        Ok((0, possibilities))
    }
}

/// Starts the CLI/shell instance
pub fn start(entries: &SongEntries) {
    println!("=== INTERACTIVE MODE ACTIVATED ===");
    println!("PRESS 'CTRL+C' TO EXIT THE PROGRAM");
    println!("TYPE 'help' FOR AVAILABLE COMMANDS");
    println!("DO NOT FORGET TO USE THE TABULATOR");

    // inspired by
    // https://github.com/trane-project/trane-cli/blob/master/src/main.rs
    let config = Config::builder()
        .auto_add_history(true)
        .color_mode(ColorMode::Enabled)
        .history_ignore_space(true)
        .completion_type(rustyline::CompletionType::List)
        .build();

    let mut rl = Editor::<ShellHelper, FileHistory>::with_config(config)
        .expect("Sorry, there's been an error!");

    let mut helper = ShellHelper::new();
    helper.complete_commands();
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
        rl.helper_mut().unwrap().complete_commands();
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
/// Prints error messages for
/// [`NotFoundError`], [`InvalidArgumentError`]
/// [`ParseError`][`chrono::format::ParseError`],
/// and [`ParseIntError`][`std::num::ParseIntError`]
#[allow(clippy::borrowed_box)]
fn handle_error(err: &Box<dyn Error>) {
    // https://users.rust-lang.org/t/matching-errorkind-from-boxed-error/30667/3
    // also thx ChatGPT
    match err.as_ref() {
        not_found if not_found.is::<NotFoundError>() => eprintln!("{not_found}"),
        invalid_arg if invalid_arg.is::<InvalidArgumentError>() => eprintln!("{invalid_arg}"),
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
        // every new command added has to have an entry in `help`!
        // and in Shellhelper.compete_commands()
        "help" | "h" => help::help(),
        "print time" | "pt" => crate::display::print_time_played(entries),
        "print time date" | "ptd" => match_print_time_date(entries, rl)?,
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
        "plot" | "g" => match_plot(entries, rl)?,
        "plot rel" | "gr" => match_plot_relative(entries, rl)?,
        "plot compare" | "gc" => match_plot_compare(entries, rl)?,
        "plot compare rel" | "gcr" => match_plot_compare_relative(entries, rl)?,
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

/// Used by [`match_input()`] for `print time date` command
fn match_print_time_date(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(), Box<dyn Error>> {
    rl.helper_mut().unwrap().reset();
    // 1st prompt: start date
    println!("Start date? YYYY-MM-DD or 'start'");
    let usr_input_start_date = rl.readline(PROMPT_SECONDARY)?;
    let start_date = user_input_date_parser(&usr_input_start_date)?;

    // 2nd prompt: end date
    println!("End date? YYYY-MM-DD or 'now'");
    let usr_input_end_date = rl.readline(PROMPT_SECONDARY)?;
    let end_date = user_input_date_parser(&usr_input_end_date)?;

    crate::display::print_time_played_date(entries, &start_date, &end_date);
    Ok(())
}

/// Used by [`match_input()`] for `print artist` command
fn match_print_artist(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(), Box<dyn Error>> {
    // prompt: artist name
    rl.helper_mut().unwrap().complete_list(entries.artists());
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
    rl.helper_mut().unwrap().complete_list(entries.artists());
    println!("Artist name?");
    let usr_input_art = rl.readline(PROMPT_MAIN)?;
    let art = entries.find().artist(&usr_input_art)?;

    rl.helper_mut().unwrap().reset();
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
    rl.helper_mut().unwrap().complete_list(entries.artists());
    println!("Artist name?");
    let usr_input_art = rl.readline(PROMPT_MAIN)?;
    let art = entries.find().artist(&usr_input_art)?;

    // 2nd prompt: album name
    rl.helper_mut().unwrap().complete_list(entries.albums(&art));
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
    rl.helper_mut().unwrap().complete_list(entries.artists());
    println!("Artist name?");
    let usr_input_art = rl.readline(PROMPT_MAIN)?;
    let art = entries.find().artist(&usr_input_art)?;

    // 2nd prompt: album name
    rl.helper_mut().unwrap().complete_list(entries.albums(&art));
    println!("Album name?");
    let usr_input_alb = rl.readline(PROMPT_MAIN)?;
    let alb = entries.find().album(&usr_input_alb, &art.name)?;

    rl.helper_mut().unwrap().reset();
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
    rl.helper_mut().unwrap().complete_list(entries.artists());
    println!("Artist name?");
    let usr_input_art = rl.readline(PROMPT_MAIN)?;
    let art = entries.find().artist(&usr_input_art)?;

    // 2nd prompt: album name
    rl.helper_mut().unwrap().complete_list(entries.albums(&art));
    println!("Album name?");
    let usr_input_alb = rl.readline(PROMPT_MAIN)?;
    let alb = entries.find().album(&usr_input_alb, &art.name)?;

    // 3rd prompt: song name
    rl.helper_mut().unwrap().complete_list(entries.songs(&alb));
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
    rl.helper_mut().unwrap().complete_list(entries.artists());
    println!("Artist name?");
    let usr_input_art = rl.readline(PROMPT_MAIN)?;
    let art = entries.find().artist(&usr_input_art)?;

    // 2nd prompt: album name
    rl.helper_mut().unwrap().complete_list(entries.albums(&art));
    println!("Album name?");
    let usr_input_alb = rl.readline(PROMPT_MAIN)?;
    let alb = entries.find().album(&usr_input_alb, &art.name)?;

    // 3rd prompt: song name
    rl.helper_mut().unwrap().complete_list(entries.songs(&alb));
    println!("Song name?");
    let usr_input_son = rl.readline(PROMPT_MAIN)?;
    let son = entries
        .find()
        .song_from_album(&usr_input_son, &alb.name, &alb.artist.name)?;

    rl.helper_mut().unwrap().reset();
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
    rl.helper_mut().unwrap().complete_list(entries.artists());
    println!("Artist name?");
    let usr_input_art = rl.readline(PROMPT_MAIN)?;
    let art = entries.find().artist(&usr_input_art)?;

    // 2nd prompt: song name
    rl.helper_mut().unwrap().complete_list(entries.songs(&art));
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
    rl.helper_mut().unwrap().complete_list(entries.artists());
    println!("Artist name?");
    let usr_input_art = rl.readline(PROMPT_MAIN)?;
    let art = entries.find().artist(&usr_input_art)?;

    // 2nd prompt: song name
    rl.helper_mut().unwrap().complete_list(entries.songs(&art));
    println!("Song name?");
    let usr_input_son = rl.readline(PROMPT_MAIN)?;
    let songs = entries.find().song(&usr_input_son, &art.name)?;

    rl.helper_mut().unwrap().reset();
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
    rl.helper_mut().unwrap().reset();
    // prompt: top n
    println!("How many Top {asp}?");
    let usr_input_n = rl.readline(PROMPT_MAIN)?;
    let num: usize = usr_input_n.parse()?;

    entries.print_top(asp, num);
    Ok(())
}

/// Used by [`match_input()`] for `plot` command
fn match_plot(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(), Box<dyn Error>> {
    // prompt: what to plot
    rl.helper_mut().unwrap().complete_aspects();
    println!("What do you want to plot? artist, album or song?");
    let usr_input_asp = rl.readline(PROMPT_SECONDARY)?;

    // other prompts
    let trace = get_absolute_trace(entries, rl, usr_input_asp.as_str())?;

    plot_single(trace);

    Ok(())
}

/// Used by [`match_input()`] for `plot relative` command
fn match_plot_relative(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(), Box<dyn Error>> {
    // prompt: what to plot
    rl.helper_mut().unwrap().complete_aspects();
    println!("What do you want to plot? artist, album or song?");
    let usr_input_asp = rl.readline(PROMPT_SECONDARY)?;

    // other prompts
    let trace = get_relative_trace(entries, rl, usr_input_asp.as_str())?;

    plot_single(trace);

    Ok(())
}

/// Used by [`match_input()`] for `plot compare` command
fn match_plot_compare(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(), Box<dyn Error>> {
    // first trace
    rl.helper_mut().unwrap().complete_aspects();
    println!("1st trace: artist, album or song?");
    let usr_input_asp_one = rl.readline(PROMPT_SECONDARY)?;
    let trace_one = get_absolute_trace(entries, rl, usr_input_asp_one.as_str())?;

    // second trace
    rl.helper_mut().unwrap().complete_aspects();
    println!("2nd trace: artist, album or song?");
    let usr_input_asp_two = rl.readline(PROMPT_SECONDARY)?;
    let trace_two = get_absolute_trace(entries, rl, usr_input_asp_two.as_str())?;

    plot_compare(trace_one, trace_two);

    Ok(())
}

/// Used by [`match_input()`] for `plot compare relative` command
fn match_plot_compare_relative(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(), Box<dyn Error>> {
    // first trace
    rl.helper_mut().unwrap().complete_aspects();
    println!("1st trace: artist, album or song?");
    let usr_input_asp_one = rl.readline(PROMPT_SECONDARY)?;
    let trace_one = get_relative_trace(entries, rl, usr_input_asp_one.as_str())?;

    // second trace
    rl.helper_mut().unwrap().complete_aspects();
    println!("2nd trace: artist, album or song?");
    let usr_input_asp_two = rl.readline(PROMPT_SECONDARY)?;
    let trace_two = get_relative_trace(entries, rl, usr_input_asp_two.as_str())?;

    plot_compare(trace_one, trace_two);

    Ok(())
}

/// Used to get traces of absolute plots
fn get_absolute_trace(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
    usr_input: &str,
) -> Result<(Box<dyn Trace>, String), Box<dyn Error>> {
    match usr_input {
        "artist" => match_plot_artist(entries, rl),
        "album" => match_plot_album(entries, rl),
        "song" => match_plot_song(entries, rl),
        _ => Err(Box::new(InvalidArgumentError::Aspect)),
    }
}

/// Used to get traces of relative plots
fn get_relative_trace(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
    usr_input: &str,
) -> Result<(Box<dyn Trace>, String), Box<dyn Error>> {
    match usr_input {
        "artist" => match_plot_artist_relative(entries, rl),
        "album" => match_plot_album_relative(entries, rl),
        "song" => match_plot_song_relative(entries, rl),
        _ => Err(Box::new(InvalidArgumentError::Aspect)),
    }
}

/// Used by [`match_plot()`] for plotting absolute plays of artist
fn match_plot_artist(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(Box<dyn Trace>, String), Box<dyn Error>> {
    // 1st prompt: artist name
    rl.helper_mut().unwrap().complete_list(entries.artists());
    println!("Artist name?");
    let usr_input_art = rl.readline(PROMPT_MAIN)?;
    let art = entries.find().artist(&usr_input_art)?;

    Ok(entries.traces().absolute(&art))
}

/// Used by [`match_plot()`] for plotting absolute plays of album
fn match_plot_album(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(Box<dyn Trace>, String), Box<dyn Error>> {
    // 1st prompt: artist name
    rl.helper_mut().unwrap().complete_list(entries.artists());
    println!("Artist name?");
    let usr_input_art = rl.readline(PROMPT_MAIN)?;
    let art = entries.find().artist(&usr_input_art)?;

    // 2nd prompt: album name
    rl.helper_mut().unwrap().complete_list(entries.albums(&art));
    println!("Album name?");
    let usr_input_alb = rl.readline(PROMPT_MAIN)?;
    let alb = entries.find().album(&usr_input_alb, &art.name)?;

    Ok(entries.traces().absolute(&alb))
}

/// Used by [`match_plot()`] for plotting absolute plays of song
fn match_plot_song(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(Box<dyn Trace>, String), Box<dyn Error>> {
    // 1st prompt: artist name
    rl.helper_mut().unwrap().complete_list(entries.artists());
    println!("Artist name?");
    let usr_input_art = rl.readline(PROMPT_MAIN)?;
    let art = entries.find().artist(&usr_input_art)?;

    // 2nd prompt: album name
    rl.helper_mut().unwrap().complete_list(entries.albums(&art));
    println!("Album name?");
    let usr_input_alb = rl.readline(PROMPT_MAIN)?;
    let alb = entries.find().album(&usr_input_alb, &art.name)?;

    // 3rd prompt: song name
    rl.helper_mut().unwrap().complete_list(entries.songs(&alb));
    println!("Song name?");
    let usr_input_son = rl.readline(PROMPT_MAIN)?;
    let son = entries
        .find()
        .song_from_album(&usr_input_son, &alb.name, &alb.artist.name)?;

    Ok(entries.traces().absolute(&son))
}

/// Used by [`match_plot_relative()`] for plotting relative plots of artist
fn match_plot_artist_relative(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(Box<dyn Trace>, String), Box<dyn Error>> {
    // 1st prompt: artist name
    rl.helper_mut().unwrap().complete_list(entries.artists());
    println!("Artist name?");
    let usr_input_art = rl.readline(PROMPT_MAIN)?;
    let art = entries.find().artist(&usr_input_art)?;

    Ok(entries.traces().relative(&art))
}

/// Used by [`match_plot_relative()`] for plotting relative plots of album
fn match_plot_album_relative(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(Box<dyn Trace>, String), Box<dyn Error>> {
    // 1st prompt: artist name
    rl.helper_mut().unwrap().complete_list(entries.artists());
    println!("Artist name?");
    let usr_input_art = rl.readline(PROMPT_MAIN)?;
    let art = entries.find().artist(&usr_input_art)?;

    // 2nd prompt: album name
    rl.helper_mut().unwrap().complete_list(entries.albums(&art));
    println!("Album name?");
    let usr_input_alb = rl.readline(PROMPT_MAIN)?;
    let alb = entries.find().album(&usr_input_alb, &art.name)?;

    // 3rd prompt: relative to what
    rl.helper_mut()
        .unwrap()
        .complete_list(vec!["all".to_string(), "artist".to_string()]);
    println!("Relative to all or artist?");
    let usr_input_rel = rl.readline(PROMPT_SECONDARY)?;

    match usr_input_rel.as_str() {
        "all" => Ok(entries.traces().relative(&alb)),
        "artist" => Ok(entries.traces().relative_to_artist(&alb)),
        _ => Err(Box::new(InvalidArgumentError::Artist)),
    }
}

/// Used by [`match_plot_relative()`] for plotting relative plots of song
fn match_plot_song_relative(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(Box<dyn Trace>, String), Box<dyn Error>> {
    // 1st prompt: artist name
    rl.helper_mut().unwrap().complete_list(entries.artists());
    println!("Artist name?");
    let usr_input_art = rl.readline(PROMPT_MAIN)?;
    let art = entries.find().artist(&usr_input_art)?;

    // 2nd prompt: album name
    rl.helper_mut().unwrap().complete_list(entries.albums(&art));
    println!("Album name?");
    let usr_input_alb = rl.readline(PROMPT_MAIN)?;
    let alb = entries.find().album(&usr_input_alb, &art.name)?;

    // 3rd prompt: song name
    rl.helper_mut().unwrap().complete_list(entries.songs(&alb));
    println!("Song name?");
    let usr_input_son = rl.readline(PROMPT_MAIN)?;
    let son = entries
        .find()
        .song_from_album(&usr_input_son, &alb.name, &alb.artist.name)?;

    // 4th prompt: relative to what
    rl.helper_mut().unwrap().complete_list(vec![
        "all".to_string(),
        "artist".to_string(),
        "album".to_string(),
    ]);

    println!("Relative to all, artist or album?");
    let usr_input_rel = rl.readline(PROMPT_SECONDARY)?;

    match usr_input_rel.as_str() {
        "all" => Ok(entries.traces().relative(&son)),
        "artist" => Ok(entries.traces().relative_to_artist(&son)),
        "album" => Ok(entries.traces().relative_to_album(&son)),
        _ => Err(Box::new(InvalidArgumentError::Album)),
    }
}

/// used by `*_date` functions in this module for when the user inputs a date
///
/// # Arguments
/// * `usr_input` - in YYYY-MM-DD format or 'now' or 'start'
pub fn user_input_date_parser(usr_input: &str) -> Result<DateTime<Tz>, chrono::format::ParseError> {
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
