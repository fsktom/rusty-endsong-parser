//! Module responsible for handling the CLI

mod help;

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;

use endsong::prelude::*;
use itertools::Itertools;
use plotly::Scatter;
use rustyline::{completion::Completer, Helper, Hinter, Validator};
use rustyline::{
    error::ReadlineError, highlight::Highlighter, history::FileHistory, ColorMode, Config, Editor,
};
use thiserror::Error;

use crate::plot;
use crate::print;
use crate::trace;
use print::{Aspect, AspectFull};
use trace::TraceType;

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

/// Error type for all errors here
#[derive(Error, Debug)]
enum UiError {
    /// Used when [`parse_date`] fails
    #[error("Invalid date! It has to be in the YYYY-MM-DD format.")]
    ParseDate(#[from] chrono::ParseError),
    /// Used when parsing user input to a number fails
    #[error("Invalid number!")]
    ParseNum(#[from] std::num::ParseIntError),
    /// Used when parsing user input to an [`Aspect`] fails
    #[error("Invalid aspect! Valid inputs: artist/s, album/s, song/s")]
    ParseAspect(#[from] print::AspectParseError),
    /// CTRL+C or similar in a main/secondary prompt, should go back to command prompt
    #[error("")]
    Readline(#[from] ReadlineError),
    /// Used when [`find`] functions return `None`
    #[error("Sorry, I couldn't find this {0} in the dataset!")]
    NotFound(&'static str),
    /// Used when user input doesn't match any comamnd
    #[error("Invalid argument! Valid inputs: {0}")]
    InvalidArgument(&'static str),
    /// Used when the end date is before the start date
    #[error("Date range is in wrong order - end date is before start date!")]
    DateWrongOrder,
}

/// Helper for [`Editor`]
#[derive(Helper, Hinter, Validator)]
struct ShellHelper {
    /// List containing all the possible completes for Tab
    completer_list: Vec<Rc<str>>,
}
impl ShellHelper {
    /// Creates a new [`ShellHelper`]
    /// with an empty tab auto-complete list
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
            "plot top",
            "plot artist albums",
        ]);
    }

    /// Changes tab-complete to ["artist", "album", "song"]
    fn complete_aspects(&mut self) {
        self.completer_list = string_vec(&["artist", "album", "song"]);
    }

    /// Changes tab-complete to the given list of valid inputs - list should be unsorted
    /// because it will be sorted here anyway
    fn complete_list(&mut self, completer_list: Vec<Rc<str>>) {
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
    type Candidate = Rc<str>;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let word = &line[0..pos];
        let possibilities = self
            .completer_list
            .iter()
            // to make the tab-complete case-insensitive
            .filter(|possible| possible.to_lowercase().starts_with(&word.to_lowercase()))
            .map(Rc::clone)
            .collect_vec();
        // assumes no escape characters...
        Ok((0, possibilities))
    }
}

/// ANSI Colors
///
/// See <https://bixense.com/clicolors>
#[derive(Debug, Copy, Clone)]
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

/// Converts a collection of [`&str`][str]s into a [`Vec<Rc<str>>`]
/// to be later used in [`ShellHelper::complete_list`]
/// for tab auto-completion
fn string_vec(slice: &[&str]) -> Vec<Rc<str>> {
    slice.iter().map(|s| Rc::from(*s)).collect_vec()
}

/// Starts the CLI/shell instance
#[allow(clippy::missing_panics_doc)]
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
                if matches!(usr_input.as_str(), "exit" | "quit" | "q") {
                    break;
                }
                match match_input(&usr_input, entries, &mut rl) {
                    Ok(()) | Err(UiError::Readline(_)) => (),
                    Err(e) => eprintln!("{e}"),
                }
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

/// Decides what to do with user input
fn match_input(
    inp: &str,
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(), UiError> {
    match inp {
        // every new command added has to have an entry in `help`!
        // and in Shellhelper::complete_commands()
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
        "plot top" | "gt" => match_plot_top(entries, rl)?,
        "plot artist albums" | "gaa" => match_plot_artist_albums(entries, rl)?,
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
) -> Result<(), UiError> {
    // 1st + 2nd prompt: start + end date
    let (start_date, end_date) = read_dates(rl)?;

    print::time_played_date(entries, &start_date, &end_date);
    Ok(())
}

/// Used by [`match_input()`] for `print max time` command
fn match_print_max_time(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(), UiError> {
    // 1st prompt: duration in days or weeks
    let valid_inputs = ["days", "weeks"];
    rl.helper_mut()
        .unwrap()
        .complete_list(string_vec(&valid_inputs));
    println!("Input time period in days or weeks?");
    let duration_type = rl.readline(PROMPT_SECONDARY)?;
    if !valid_inputs.iter().any(|&s| s == duration_type) {
        return Err(UiError::InvalidArgument("days, weeks"));
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
) -> Result<(), UiError> {
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
) -> Result<(), UiError> {
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
) -> Result<(), UiError> {
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
) -> Result<(), UiError> {
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
) -> Result<(), UiError> {
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
) -> Result<(), UiError> {
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
) -> Result<(), UiError> {
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
) -> Result<(), UiError> {
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
            gather::plays_of_many(entries.between(&start_date, &end_date), &songs)
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
) -> Result<(), UiError> {
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
) -> Result<(), UiError> {
    // prompt: what to plot
    rl.helper_mut().unwrap().complete_aspects();
    println!("What do you want to plot? artist, album or song?");
    let usr_input_asp = rl.readline(PROMPT_SECONDARY)?;

    // other prompts
    let (trace, title) = get_absolute_trace(entries, rl, usr_input_asp.as_str())?;

    plot::single((TraceType::Absolute(trace), title));

    Ok(())
}

/// Used by [`match_input()`] for `plot relative` command
fn match_plot_relative(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(), UiError> {
    // prompt: what to plot
    rl.helper_mut().unwrap().complete_aspects();
    println!("What do you want to plot? artist, album or song?");
    let usr_input_asp = rl.readline(PROMPT_SECONDARY)?;

    // other prompts
    let (trace, title) = get_relative_trace(entries, rl, usr_input_asp.as_str())?;

    plot::single((TraceType::Relative(trace), title));

    Ok(())
}

/// Used by [`match_input()`] for `plot compare` command
fn match_plot_compare(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(), UiError> {
    // first trace
    rl.helper_mut().unwrap().complete_aspects();
    println!("1st trace: artist, album or song?");
    let usr_input_asp_one = rl.readline(PROMPT_SECONDARY)?;
    let (trace_one, title_one) = get_absolute_trace(entries, rl, usr_input_asp_one.as_str())?;

    // second trace
    rl.helper_mut().unwrap().complete_aspects();
    println!("2nd trace: artist, album or song?");
    let usr_input_asp_two = rl.readline(PROMPT_SECONDARY)?;
    let (trace_two, title_two) = get_absolute_trace(entries, rl, usr_input_asp_two.as_str())?;

    plot::compare(
        (TraceType::Absolute(trace_one), title_one),
        (TraceType::Absolute(trace_two), title_two),
    );

    Ok(())
}

/// Used by [`match_input()`] for `plot compare relative` command
fn match_plot_compare_relative(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(), UiError> {
    // first trace
    rl.helper_mut().unwrap().complete_aspects();
    println!("1st trace: artist, album or song?");
    let usr_input_asp_one = rl.readline(PROMPT_SECONDARY)?;
    let (trace_one, title_one) = get_relative_trace(entries, rl, usr_input_asp_one.as_str())?;

    // second trace
    rl.helper_mut().unwrap().complete_aspects();
    println!("2nd trace: artist, album or song?");
    let usr_input_asp_two = rl.readline(PROMPT_SECONDARY)?;
    let (trace_two, title_two) = get_relative_trace(entries, rl, usr_input_asp_two.as_str())?;

    plot::compare(
        (TraceType::Relative(trace_one), title_one),
        (TraceType::Relative(trace_two), title_two),
    );

    Ok(())
}

/// Used by [`match_input()`] for `plot top` command
fn match_plot_top(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(), UiError> {
    // prompt: what to plot
    rl.helper_mut().unwrap().complete_aspects();
    println!("What do you want to plot? Top artists, albums or songs?");
    let usr_input_asp = rl.readline(PROMPT_MAIN)?;
    let aspect: Aspect = usr_input_asp.parse()?;

    // prompt: top n
    rl.helper_mut().unwrap().reset();
    println!("How many top {aspect} to plot? (recommended: ~5)");
    let usr_input_n = rl.readline(PROMPT_SECONDARY)?;
    let num: usize = usr_input_n.parse()?;

    // TODO prompt: sum songs from different albums?

    let traces = match aspect {
        Aspect::Artists => get_traces(entries, &gather::artists(entries), num),
        Aspect::Albums => get_traces(entries, &gather::albums(entries), num),
        Aspect::Songs => get_traces(entries, &gather::songs(entries, true), num),
    };

    plot::multiple(traces, &format!("Top {aspect}"));

    Ok(())
}

/// Used by [`match_input()`] for `plot artist albums` command
fn match_plot_artist_albums(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(), UiError> {
    // prompt: artist name
    let art = read_artist(rl, entries)?;

    let albums_map = gather::albums_from_artist(entries, &art);
    let albums = albums_map
        .iter()
        .sorted_unstable_by_key(|t| (std::cmp::Reverse(t.1), t.0))
        .map(|(aspect, _)| aspect)
        .collect_vec();

    let mut traces = vec![];
    for (count, alb) in albums.into_iter().enumerate() {
        let TraceType::Absolute(trace) = trace::absolute(entries, alb) else {
            unreachable!()
        };

        let trace = trace
            .legend_group_title(art.name.to_string())
            .name(&alb.name);

        // only the traces for the 3 albums with most plays are shown by default
        let trace = if count < 3 {
            trace
        } else {
            // others are hidden and have to be enabled manually
            trace.visible(plotly::common::Visible::LegendOnly)
        };

        traces.push(TraceType::Absolute(trace));
    }

    let title = format!("{art} albums");

    plot::multiple(traces, &title);

    Ok(())
}

/// Returns the traces for the top `num` artists, albums or songs
///
/// Helper function for [`match_plot_top`]
fn get_traces<Asp: Music>(
    entries: &SongEntries,
    music_map: &HashMap<Asp, usize>,
    num: usize,
) -> Vec<TraceType> {
    music_map
        .iter()
        .sorted_unstable_by_key(|t| (std::cmp::Reverse(t.1), t.0))
        .take(num)
        .map(|(aspect, _)| trace::absolute(entries, aspect))
        .collect_vec()
}

/// Used to get traces of absolute plots
fn get_absolute_trace(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
    usr_input: &str,
) -> Result<(Box<Scatter<String, usize>>, String), UiError> {
    match usr_input {
        "artist" => match_plot_artist(entries, rl),
        "album" => match_plot_album(entries, rl),
        "song" => match_plot_song(entries, rl),
        _ => Err(UiError::InvalidArgument("artist, album, song")),
    }
}

/// Used to get traces of relative plots
fn get_relative_trace(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
    usr_input: &str,
) -> Result<(Box<Scatter<String, f64>>, String), UiError> {
    match usr_input {
        "artist" => match_plot_artist_relative(entries, rl),
        "album" => match_plot_album_relative(entries, rl),
        "song" => match_plot_song_relative(entries, rl),
        _ => Err(UiError::InvalidArgument("artist, album, song")),
    }
}

/// Used by [`match_plot()`] for plotting absolute plays of artist
fn match_plot_artist(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(Box<Scatter<String, usize>>, String), UiError> {
    // 1st prompt: artist name
    let art = read_artist(rl, entries)?;

    if let TraceType::Absolute(trace) = trace::absolute(entries, &art) {
        Ok((trace, art.to_string()))
    } else {
        unreachable!()
    }
}

/// Used by [`match_plot()`] for plotting absolute plays of album
fn match_plot_album(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(Box<Scatter<String, usize>>, String), UiError> {
    // 1st prompt: artist name
    let art = read_artist(rl, entries)?;

    // 2nd prompt: album name
    let alb = read_album(rl, entries, &art)?;

    if let TraceType::Absolute(trace) = trace::absolute(entries, &alb) {
        Ok((trace, alb.to_string()))
    } else {
        unreachable!()
    }
}

/// Used by [`match_plot()`] for plotting absolute plays of song
fn match_plot_song(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(Box<Scatter<String, usize>>, String), UiError> {
    // 1st prompt: artist name
    let art = read_artist(rl, entries)?;

    // 2nd prompt: album name
    let alb = read_album(rl, entries, &art)?;

    // 3rd prompt: song name
    let son = read_song(rl, entries, &alb)?;

    if let TraceType::Absolute(trace) = trace::absolute(entries, &son) {
        Ok((trace, son.to_string()))
    } else {
        unreachable!()
    }
}

/// Used by [`match_plot_relative()`] for plotting relative plots of artist
fn match_plot_artist_relative(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(Box<Scatter<String, f64>>, String), UiError> {
    // 1st prompt: artist name
    let art = read_artist(rl, entries)?;

    let trace = trace::relative::to_all(entries, &art);

    if let TraceType::Relative(trace) = trace {
        Ok((trace, art.to_string()))
    } else {
        unreachable!()
    }
}

/// Used by [`match_plot_relative()`] for plotting relative plots of album
fn match_plot_album_relative(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(Box<Scatter<String, f64>>, String), UiError> {
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

    let trace = match usr_input_rel.as_str() {
        "all" => trace::relative::to_all(entries, &alb),
        "artist" => trace::relative::to_artist(entries, &alb),
        _ => return Err(UiError::InvalidArgument("all, artist")),
    };

    if let TraceType::Relative(trace) = trace {
        Ok((trace, alb.to_string()))
    } else {
        unreachable!()
    }
}

/// Used by [`match_plot_relative()`] for plotting relative plots of song
fn match_plot_song_relative(
    entries: &SongEntries,
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(Box<Scatter<String, f64>>, String), UiError> {
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

    let trace = match usr_input_rel.as_str() {
        "all" => trace::relative::to_all(entries, &son),
        "artist" => trace::relative::to_artist(entries, &son),
        "album" => trace::relative::to_album(entries, &son),
        _ => return Err(UiError::InvalidArgument("all, artist, album")),
    };

    if let TraceType::Relative(trace) = trace {
        Ok((trace, son.to_string()))
    } else {
        unreachable!()
    }
}

/// Used by `*_date` functions for reading start and end dates from user
///
/// Returns `(start_date, end_date)`
fn read_dates(
    rl: &mut Editor<ShellHelper, FileHistory>,
) -> Result<(DateTime<Local>, DateTime<Local>), UiError> {
    // make sure no wrong autocompletes appear
    rl.helper_mut().unwrap().reset();

    // 1st prompt: start date
    println!("Start date? YYYY-MM-DD or 'start'");
    let usr_input_start_date = rl.readline(PROMPT_SECONDARY)?;
    let start_date = parse_date(&usr_input_start_date)?;

    // 2nd prompt: end date
    println!("End date? YYYY-MM-DD or 'now'");
    let usr_input_end_date = rl.readline(PROMPT_SECONDARY)?;
    let end_date = parse_date(&usr_input_end_date)?;

    if start_date >= end_date {
        return Err(UiError::DateWrongOrder);
    }
    Ok((start_date, end_date))
}

/// Used by `match_*` functions for finding [`Artist`] from user input
fn read_artist(
    rl: &mut Editor<ShellHelper, FileHistory>,
    entries: &SongEntries,
) -> Result<Artist, UiError> {
    // prompt: artist name
    rl.helper_mut().unwrap().complete_list(entries.artists());
    println!("Artist name?");
    let usr_input_art = rl.readline(PROMPT_MAIN)?;
    entries
        .find()
        .artist(&usr_input_art)
        .ok_or(UiError::NotFound("artist"))
}

/// Used by `match_*` functions for finding [`Album`] from user input
fn read_album(
    rl: &mut Editor<ShellHelper, FileHistory>,
    entries: &SongEntries,
    art: &Artist,
) -> Result<Album, UiError> {
    // prompt: album name
    rl.helper_mut().unwrap().complete_list(entries.albums(art));
    println!("Album name?");
    let usr_input_alb = rl.readline(PROMPT_MAIN)?;
    entries
        .find()
        .album(&usr_input_alb, &art.name)
        .ok_or(UiError::NotFound("album from this artist"))
}

/// Used by `match_*` functions for finding [`Song`] from user input
fn read_song(
    rl: &mut Editor<ShellHelper, FileHistory>,
    entries: &SongEntries,
    alb: &Album,
) -> Result<Song, UiError> {
    // prompt: song name
    rl.helper_mut().unwrap().complete_list(entries.songs(alb));
    println!("Song name?");
    let usr_input_son = rl.readline(PROMPT_MAIN)?;
    entries
        .find()
        .song_from_album(&usr_input_son, &alb.name, &alb.artist.name)
        .ok_or(UiError::NotFound("song from this album"))
}

/// Used by `match_*` functions for finding [`Vec<Song>`] from user input
fn read_songs(
    rl: &mut Editor<ShellHelper, FileHistory>,
    entries: &SongEntries,
    art: &Artist,
) -> Result<Vec<Song>, UiError> {
    // prompt: song name
    rl.helper_mut().unwrap().complete_list(entries.songs(art));
    println!("Song name?");
    let usr_input_son = rl.readline(PROMPT_MAIN)?;
    entries
        .find()
        .song(&usr_input_son, &art.name)
        .ok_or(UiError::NotFound("song from this artist"))
}
