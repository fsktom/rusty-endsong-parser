//! Module responsible for handling the CLI

mod help;

use std::borrow::Cow;
use std::error::Error;
use std::fmt::Display;

use chrono::{DateTime, Duration, TimeZone};
use chrono_tz::Tz;
use plotly::Trace;
use rustyline::{completion::Completer, Helper, Hinter, Validator};
use rustyline::{
    error::ReadlineError, highlight::Highlighter, history::FileHistory, ColorMode, Config, Editor,
};

use crate::plot;
use crate::print;
use crate::types::SongEntries;
use crate::types::{Album, Artist, Song};
use crate::LOCATION_TZ;
use print::{Aspect, AspectFull};

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
    /// Error message: Date range is in wrong order - start date is after end date!
    DateWrongOrder,
    /// Error message: Invalid argument! Try using 'weeks' or 'days' next time
    DurationType,
}
impl Display for InvalidArgumentError {
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
            InvalidArgumentError::DateWrongOrder => write!(
                f,
                "Date range is in wrong order - start date is after end date!"
            ),
            InvalidArgumentError::DurationType => write!(
                f,
                " Invalid argument! Try using 'weeks' or 'days' next time"
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
        self.completer_list = string_vec(&[
            "help",
            "print time",
            "print time date",
            "print max time",
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
        ]);
    }

    /// Changes tab-complete to ["artist", "album", "song"]
    fn complete_aspects(&mut self) {
        self.completer_list = string_vec(&["artist", "album", "song"]);
    }

    /// Changes tab-complete to the given list of valid inputs - list should be unsorted
    fn complete_list(&mut self, completer_list: Vec<String>) {
        self.completer_list = completer_list;
        self.completer_list.sort_unstable();
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

/// ANSI Colors
///
/// See <https://bixense.com/clicolors>
enum Color {
    /// Resets the following text with `\x1b[0m`
    Reset,
    /// Makes the following text green with `\x1b[1;32m`
    Green,
    /// Makes the following text light green with `\x1b[0;32m`
    LightGreen,
    /// Makes the following text cyan with `\x1b[1;36m`
    Cyan,
    /// Makes the following text red with `\x1b[1;31m`
    Red,
    /// Makes the following text pink with `\x1b[1;35m`
    Pink,
}
impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::Reset => write!(f, "\x1b[0m"),
            Color::Green => write!(f, "\x1b[1;32m"),
            Color::LightGreen => write!(f, "\x1b[0;32m"),
            Color::Cyan => write!(f, "\x1b[1;36m"),
            Color::Red => write!(f, "\x1b[1;31m"),
            Color::Pink => write!(f, "\x1b[1;35m"),
        }
    }
}

/// Errors if [`Artist`], [`Album`] or [`Song`] are not found
/// with custom error messages
#[derive(Debug)]
pub enum NotFoundError {
    /// Artist with that name was not found
    ///
    /// Error message: "Sorry, I couldn't find any artist with that name!"
    Artist,
    /// Album with that name from that artist was not found
    ///
    /// Error message: "Sorry, I couldn't find any album with that name
    /// from that artist!"
    Album,
    /// Song with that name from that album and artist was not found
    ///
    /// Error message:
    /// "Sorry, I couldn't find any song with
    /// that name from that album and artist!"
    Song,
}
impl Display for NotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotFoundError::Artist => {
                write!(f, "Sorry, I couldn't find any artist with that name!")
            }
            NotFoundError::Album => {
                write!(
                    f,
                    "Sorry, I couldn't find any album with that name from that artist!"
                )
            }
            NotFoundError::Song => {
                write!(
                    f,
                    "Sorry, I couldn't find any song with that name from that album and artist!"
                )
            }
        }
    }
}
impl Error for NotFoundError {}

/// Converts a collection of [`&str`][str]s into a [`Vec<String>`]
fn string_vec(slice: &[&str]) -> Vec<String> {
    slice
        .iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>()
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
    if !history_path.try_exists().unwrap() {
        if let Err(e) = std::fs::File::create(history_path) {
            eprintln!("Failed to create history file: {e}");
        }
    }
    if let Err(e) = rl.load_history(history_path) {
        eprintln!(
            "Failed to load history file at {}: {e}",
            history_path.to_str().unwrap()
        );
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
                eprintln!("CTRL+D - execution has stopped!");
                break;
            }
            Err(err) => {
                eprintln!("Execution has stopped! - Error: {err}");
                break;
            }
        }
        rl.helper_mut().unwrap().complete_commands();
    }

    if let Err(e) = rl.save_history(history_path) {
        eprintln!(
            "Failed to save history to file {}: {e}",
            history_path.to_str().unwrap()
        );
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
        "print time" | "pt" => print::time_played(entries),
        "print time date" | "ptd" => match_print_time_date(entries, rl)?,
        "print max time" | "pmt" => match_print_max_time(entries, rl)?,
        "print artist" | "part" => match_print_artist(entries, rl)?,
        "print album" | "palb" => match_print_album(entries, rl)?,
        "print song" | "pson" => match_print_song(entries, rl)?,
        "print songs" | "psons" => match_print_songs(entries, rl)?,
        "print artist date" | "partd" => match_print_artist_date(entries, rl)?,
        "print album date" | "palbd" => match_print_album_date(entries, rl)?,
        "print song date" | "psond" => match_print_song_date(entries, rl)?,
        "print songs date" | "psonsd" => match_print_songs_date(entries, rl)?,
        "print top artists" | "ptarts" => match_print_top(entries, rl, Aspect::Artists, false)?,
        "print top albums" | "ptalbs" => match_print_top(entries, rl, Aspect::Albums, false)?,
        "print top songs" | "ptsons" => match_print_top(entries, rl, Aspect::Songs, true)?,
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
    // 1st + 2nd prompt: start + end date
    let (start_date, end_date) = read_dates(rl)?;

    print::time_played_date(entries, &start_date, &end_date);
    Ok(())
}

/// Used by [`match_input()`] for `print max time` command
fn match_print_max_time(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(), Box<dyn Error>> {
    // 1st prompt: duration in days or weeks
    let valid_inputs = vec!["days".to_string(), "weeks".to_string()];
    rl.helper_mut().unwrap().complete_list(valid_inputs.clone());
    println!("Input time period in days or weeks?");
    let duration_type = rl.readline(PROMPT_SECONDARY)?;
    if !valid_inputs.contains(&duration_type) {
        return Err(Box::new(InvalidArgumentError::DurationType));
    };

    rl.helper_mut().unwrap().reset();
    // 2nd prompt: actual duration number
    println!("What's the time period? Whole numbers only");
    let usr_input_duration = rl.readline(PROMPT_SECONDARY)?;
    let duration_num = usr_input_duration.parse::<i64>()?;

    let (_, start, end) = match duration_type.as_str() {
        "days" => entries.max_listening_time(Duration::days(duration_num)),
        "weeks" => entries.max_listening_time(Duration::weeks(duration_num)),
        // is unreachable because of the check above
        _ => unreachable!(),
    };

    // temporary, maybe later make a custom one
    print::time_played_date(entries, &start, &end);

    Ok(())
}

/// Used by [`match_input()`] for `print artist` command
fn match_print_artist(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(), Box<dyn Error>> {
    // prompt: artist name
    let art = read_artist(rl, entries)?;

    print::aspect(entries, &AspectFull::Artist(&art));
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
    let art = read_artist(rl, entries)?;

    // 2nd + 3rd prompt: start + end date
    let (start_date, end_date) = read_dates(rl)?;

    print::aspect_date(entries, &AspectFull::Artist(&art), &start_date, &end_date);
    Ok(())
}

/// Used by [`match_input()`] for `print album` command
fn match_print_album(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(), Box<dyn Error>> {
    // 1st prompt: artist name
    let art = read_artist(rl, entries)?;

    // 2nd prompt: album name
    let alb = read_album(rl, entries, &art)?;

    print::aspect(entries, &AspectFull::Album(&alb));
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
    let art = read_artist(rl, entries)?;

    // 2nd prompt: album name
    let alb = read_album(rl, entries, &art)?;

    // 3rd + 4th prompt: start + end date
    let (start_date, end_date) = read_dates(rl)?;

    print::aspect_date(entries, &AspectFull::Album(&alb), &start_date, &end_date);
    Ok(())
}

/// Used by [`match_input()`] for `print song` command
fn match_print_song(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(), Box<dyn Error>> {
    // 1st prompt: artist name
    let art = read_artist(rl, entries)?;

    // 2nd prompt: album name
    let alb = read_album(rl, entries, &art)?;

    // 3rd prompt: song name
    let son = read_song(rl, entries, &alb)?;

    print::aspect(entries, &AspectFull::Song(&son));
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
    let art = read_artist(rl, entries)?;

    // 2nd prompt: album name
    let alb = read_album(rl, entries, &art)?;

    // 3rd prompt: song name
    let son = read_song(rl, entries, &alb)?;

    // 4th + 5th prompt: start + end date
    let (start_date, end_date) = read_dates(rl)?;

    print::aspect_date(entries, &AspectFull::Song(&son), &start_date, &end_date);
    Ok(())
}

/// Used by [`match_input()`] for `print songs` command
fn match_print_songs(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(), Box<dyn Error>> {
    // 1st prompt: artist name
    let art = read_artist(rl, entries)?;

    // 2nd prompt: song name
    let songs = read_songs(rl, entries, &art)?;

    // if there are multiple songs with that name found
    if songs.len() > 1 {
        println!(
            "I've found {} songs named {} from {} with a total of {} plays!",
            songs.len(),
            &songs[0].name,
            &songs[0].album.artist.name,
            entries.gather_plays_of_many(&songs)
        );
    }
    for song in songs {
        print::aspect(entries, &AspectFull::Song(&song));
    }
    Ok(())
}

/// Used by [`match_input()`] for `print songs date` command
fn match_print_songs_date(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(), Box<dyn Error>> {
    // 1st prompt: artist name
    let art = read_artist(rl, entries)?;

    // 2nd prompt: song name
    let songs = read_songs(rl, entries, &art)?;

    // 3rd + 4th prompt: start + end date
    let (start_date, end_date) = read_dates(rl)?;

    // if there are multiple songs with that name found
    if songs.len() > 1 {
        println!(
            "I've found {} songs named {} from {} with a total of {} plays!",
            songs.len(),
            &songs[0].name,
            &songs[0].album.artist.name,
            entries.gather_plays_of_many_date(&songs, &start_date, &end_date)
        );
    }
    for song in songs {
        print::aspect_date(entries, &AspectFull::Song(&song), &start_date, &end_date);
    }

    Ok(())
}

/// Used by [`match_input()`] for `print top artists/albums/songs` commands
fn match_print_top(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
    asp: Aspect,
    ask_for_sum: bool,
) -> Result<(), Box<dyn Error>> {
    rl.helper_mut().unwrap().reset();
    // prompt: top n
    println!("How many Top {asp}?");
    let usr_input_n = rl.readline(PROMPT_MAIN)?;
    let num: usize = usr_input_n.parse()?;

    let mut sum_songs_from_different_albums = false;
    if ask_for_sum {
        // prompt: ask if you want to sum songs from different albums
        rl.helper_mut()
            .unwrap()
            .complete_list(string_vec(&["yes", "y", "no", "n"]));
        println!("Do you want to sum songs from different albums? (y/n)");
        let usr_input_b = rl.readline(PROMPT_SECONDARY)?;
        sum_songs_from_different_albums = match usr_input_b.as_str() {
            "yes" | "y" => true,
            "no" | "n" => false,
            _ => {
                println!("Invalid input. Assuming 'no'.");
                false
            }
        }
    }

    print::top(entries, asp, num, sum_songs_from_different_albums);
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

    plot::single(trace);

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

    plot::single(trace);

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

    plot::compare(trace_one, trace_two);

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

    plot::compare(trace_one, trace_two);

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
    let art = read_artist(rl, entries)?;

    Ok(plot::absolute::aspect(entries, &art))
}

/// Used by [`match_plot()`] for plotting absolute plays of album
fn match_plot_album(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(Box<dyn Trace>, String), Box<dyn Error>> {
    // 1st prompt: artist name
    let art = read_artist(rl, entries)?;

    // 2nd prompt: album name
    let alb = read_album(rl, entries, &art)?;

    Ok(plot::absolute::aspect(entries, &alb))
}

/// Used by [`match_plot()`] for plotting absolute plays of song
fn match_plot_song(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(Box<dyn Trace>, String), Box<dyn Error>> {
    // 1st prompt: artist name
    let art = read_artist(rl, entries)?;

    // 2nd prompt: album name
    let alb = read_album(rl, entries, &art)?;

    // 3rd prompt: song name
    let son = read_song(rl, entries, &alb)?;

    Ok(plot::absolute::aspect(entries, &son))
}

/// Used by [`match_plot_relative()`] for plotting relative plots of artist
fn match_plot_artist_relative(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(Box<dyn Trace>, String), Box<dyn Error>> {
    // 1st prompt: artist name
    let art = read_artist(rl, entries)?;

    Ok(plot::relative::to_all(entries, &art))
}

/// Used by [`match_plot_relative()`] for plotting relative plots of album
fn match_plot_album_relative(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(Box<dyn Trace>, String), Box<dyn Error>> {
    // 1st prompt: artist name
    let art = read_artist(rl, entries)?;

    // 2nd prompt: album name
    let alb = read_album(rl, entries, &art)?;

    // 3rd prompt: relative to what
    rl.helper_mut()
        .unwrap()
        .complete_list(string_vec(&["all", "artist"]));
    println!("Relative to all or artist?");
    let usr_input_rel = rl.readline(PROMPT_SECONDARY)?;

    match usr_input_rel.as_str() {
        "all" => Ok(plot::relative::to_all(entries, &alb)),
        "artist" => Ok(plot::relative::to_artist(entries, &alb)),
        _ => Err(Box::new(InvalidArgumentError::Artist)),
    }
}

/// Used by [`match_plot_relative()`] for plotting relative plots of song
fn match_plot_song_relative(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(Box<dyn Trace>, String), Box<dyn Error>> {
    // 1st prompt: artist name
    let art = read_artist(rl, entries)?;

    // 2nd prompt: album name
    let alb = read_album(rl, entries, &art)?;

    // 3rd prompt: song name
    let son = read_song(rl, entries, &alb)?;

    // 4th prompt: relative to what
    rl.helper_mut()
        .unwrap()
        .complete_list(string_vec(&["all", "artist", "album"]));

    println!("Relative to all, artist or album?");
    let usr_input_rel = rl.readline(PROMPT_SECONDARY)?;

    match usr_input_rel.as_str() {
        "all" => Ok(plot::relative::to_all(entries, &son)),
        "artist" => Ok(plot::relative::to_artist(entries, &son)),
        "album" => Ok(plot::relative::to_album(entries, &son)),
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
        // -> problem with that: would have to pass &entries to this function
        // actually not big problem, I could even put LOCATION_TZ as a field of it
        // and not a constant :O
        "start" => String::from("1980-01-01T00:00:00Z"),
        // usr_input should be in YYYY-MM-DD format
        _ => format!("{usr_input}T00:00:00Z"),
    };

    // "%FT%TZ" is equivalent to "%Y-%m-%dT%H:%M:%SZ"
    // see <https://docs.rs/chrono/latest/chrono/format/strftime/index.html>
    LOCATION_TZ.datetime_from_str(&date_str, "%FT%TZ")
}

/// Used by `*_date()` functions for reading start and end dates from user
///
/// Returns `(start_date, end_date)`
fn read_dates(
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(DateTime<Tz>, DateTime<Tz>), Box<dyn Error>> {
    // make sure no wrong autocompletes appear
    rl.helper_mut().unwrap().reset();

    // 1st prompt: start date
    println!("Start date? YYYY-MM-DD or 'start'");
    let usr_input_start_date = rl.readline(PROMPT_SECONDARY)?;
    let start_date = user_input_date_parser(&usr_input_start_date)?;

    // 2nd prompt: end date
    println!("End date? YYYY-MM-DD or 'now'");
    let usr_input_end_date = rl.readline(PROMPT_SECONDARY)?;
    let end_date = user_input_date_parser(&usr_input_end_date)?;

    if start_date >= end_date {
        return Err(Box::new(InvalidArgumentError::DateWrongOrder));
    }
    Ok((start_date, end_date))
}

/// Used by `match_*` functions for finding [`Artist`] from user input
fn read_artist(
    rl: &mut Editor<ShellHelper, FileHistory>,
    entries: &SongEntries,
) -> Result<Artist, Box<dyn Error>> {
    // prompt: artist name
    rl.helper_mut().unwrap().complete_list(entries.artists());
    println!("Artist name?");
    let usr_input_art = rl.readline(PROMPT_MAIN)?;
    entries
        .find()
        .artist(&usr_input_art)
        .ok_or(Box::new(NotFoundError::Artist))
}

/// Used by `match_*` functions for finding [`Album`] from user input
fn read_album(
    rl: &mut Editor<ShellHelper, FileHistory>,
    entries: &SongEntries,
    art: &Artist,
) -> Result<Album, Box<dyn Error>> {
    // prompt: album name
    rl.helper_mut().unwrap().complete_list(entries.albums(art));
    println!("Album name?");
    let usr_input_alb = rl.readline(PROMPT_MAIN)?;
    entries
        .find()
        .album(&usr_input_alb, &art.name)
        .ok_or(Box::new(NotFoundError::Album))
}

/// Used by `match_*` functions for finding [`Song`] from user input
fn read_song(
    rl: &mut Editor<ShellHelper, FileHistory>,
    entries: &SongEntries,
    alb: &Album,
) -> Result<Song, Box<dyn Error>> {
    // prompt: song name
    rl.helper_mut().unwrap().complete_list(entries.songs(alb));
    println!("Song name?");
    let usr_input_son = rl.readline(PROMPT_MAIN)?;
    entries
        .find()
        .song_from_album(&usr_input_son, &alb.name, &alb.artist.name)
        .ok_or(Box::new(NotFoundError::Song))
}

/// Used by `match_*` functions for finding [`Vec<Song>`] from user input
fn read_songs(
    rl: &mut Editor<ShellHelper, FileHistory>,
    entries: &SongEntries,
    art: &Artist,
) -> Result<Vec<Song>, Box<dyn Error>> {
    // prompt: song name
    rl.helper_mut().unwrap().complete_list(entries.songs(art));
    println!("Song name?");
    let usr_input_son = rl.readline(PROMPT_MAIN)?;
    entries
        .find()
        .song(&usr_input_son, &art.name)
        .ok_or(Box::new(NotFoundError::Song))
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
