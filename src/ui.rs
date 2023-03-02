//! Module responsible for handling the CLI

use crate::types::{AspectFull, SongEntries};
use crate::LOCATION_TZ;

use std::collections::HashMap;
use std::error::Error;

use chrono::{DateTime, TimeZone};
use chrono_tz::Tz;
use rustyline::{error::ReadlineError, ColorMode, Config, Editor};

/// Prompt used for top-level shell commands
///
/// green `>>>`
// https://bixense.com/clicolors/
// \x1b[1;32m makes ">>>" green
// \x1b[0m makes user input default color again
const PROMPT_COMMAND: &str = "\x1b[1;32m>>>\x1b[0m ";

/// Prompt used for main arguments like artist, album and song name
///
/// cyan `  >>`
// \x1b[1;36m makes ">>" cyan
// \x1b[0m makes user input default color again
const PROMPT_MAIN: &str = "  \x1b[1;36m>>\x1b[0m ";

/// Prompt used for additional arguments like the date range
///
/// red `   >`
// \x1b[1;31m makes ">" red
// \x1b[0m makes user input default color again
const PROMPT_SECONDARY: &str = "   \x1b[1;31m>\x1b[0m ";

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

    let mut rl = Editor::<()>::with_config(config).expect("Sorry, there's been an error!");

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
                match_input(&usr_input, entries, &mut rl).unwrap_or_else(|e| println!("{e}"));
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {err:?}");
                break;
            }
        }
    }

    match rl.save_history(history_path) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Failed to save history to file .trane_history: {e}");
        }
    }
}

/// Decides what to do with user input
fn match_input(
    inp: &str,
    entries: &SongEntries,
    rl: &mut Editor<()>,
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
        // when you press ENTER -> nothing happens, new prompt
        "" => (),
        _ => {
            // \x1b[1;31m makes text red
            // \x1b[0m makes it the default color
            println!("Command not found! Type \x1b[1;31mhelp\x1b[0m to print available commands");
        }
    }
    Ok(())
}

/// Used by [`match_input`()] for `print artist` command
fn match_print_artist(entries: &SongEntries, rl: &mut Editor<()>) -> Result<(), Box<dyn Error>> {
    // prompt: artist name
    println!("Artist name?");
    let usr_input_art = rl.readline(PROMPT_MAIN)?;
    let art = entries.find().artist(&usr_input_art)?;

    entries.print_aspect(&AspectFull::Artist(&art));
    Ok(())
}

/// Used by [`match_input`()] for `print artist date` command
///
/// Basically [`match_print_artist`()] but with date functionality
fn match_print_artist_date(
    entries: &SongEntries,
    rl: &mut Editor<()>,
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

/// Used by [`match_input`()] for `print album` command
fn match_print_album(entries: &SongEntries, rl: &mut Editor<()>) -> Result<(), Box<dyn Error>> {
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

/// Used by [`match_input`()] for `print album date` command
///
/// Basically [`match_print_album`()] but with date functionality
fn match_print_album_date(
    entries: &SongEntries,
    rl: &mut Editor<()>,
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

/// Used by [`match_input`()] for `print song` command
fn match_print_song(entries: &SongEntries, rl: &mut Editor<()>) -> Result<(), Box<dyn Error>> {
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

/// Used by [`match_input`()] for `print song date` command
///
/// Basically [`match_print_song`()] but with date functionality
fn match_print_song_date(entries: &SongEntries, rl: &mut Editor<()>) -> Result<(), Box<dyn Error>> {
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

/// Used by [`match_input`()] for `print songs` command
fn match_print_songs(entries: &SongEntries, rl: &mut Editor<()>) -> Result<(), Box<dyn Error>> {
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

/// Used by [`match_input`()] for `print songs date` command
fn match_print_songs_date(
    entries: &SongEntries,
    rl: &mut Editor<()>,
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

/// Used by [`match_input`()] for `help` command
///
/// Prints the available commands to the [`std::io::stdout`]
fn help() {
    // alias in pink! \x1b[1;35m
    // actual command in red! \x1b[1;31m
    // reset color with \x1b[0m

    let mut meta_commands: HashMap<&str, &str> = HashMap::new();
    meta_commands.insert(
        "help",
        "prints this command list
        \t\x1b[1;35malias: h\x1b[0m",
    );

    let mut print_commands: HashMap<&str, &str> = HashMap::new();
    print_commands.insert(
        "print artist",
        "prints every album from the artist
        \topens another prompt where you input the artist name
        \t\x1b[1;35malias: part\x1b[0m",
    );
    print_commands.insert(
        "print album",
        "prints every song from the album
        \topens another prompt where you input the artist name
        \tand then the album name
        \t\x1b[1;35malias: palb\x1b[0m",
    );
    print_commands.insert(
        "print song",
        "prints a song
        \topens another prompt where you input the artist name
        \tand then the album name
        \tand then the song name
        \t\x1b[1;35malias: pson\x1b[0m",
    );
    print_commands.insert(
        "print songs",
        "prints a song with all the albums it may be from
        \topens another prompt where you input the artist name
        \tand then the song name
        \t\x1b[1;35malias: psons\x1b[0m",
    );

    print_commands.insert(
        "print artist date",
        "prints every album from the artist within a date range
        \topens another prompt where you input the artist name
        \tand then the date range
        \t\x1b[1;35malias: partd\x1b[0m",
    );
    print_commands.insert(
        "print album date",
        "prints every song from the album within a date range
        \topens another prompt where you input the artist name
        \tand then the album name
        \t\x1b[1;35malias: palbd\x1b[0m",
    );
    print_commands.insert(
        "print song date",
        "prints a song within a date range
        \topens another prompt where you input the artist name
        \tand then the album name
        \tand then the song name
        \tand then the date range
        \t\x1b[1;35malias: psond\x1b[0m",
    );
    print_commands.insert(
        "print songs date",
        "prints a song with all the albums it may be from within a date range
        \topens another prompt where you input the artist name
        \tand then the song name
        \tand then the date range
        \t\x1b[1;35malias: psonsd\x1b[0m",
    );

    let mut graph_commands: HashMap<&str, &str> = HashMap::new();
    graph_commands.insert(
        "graph placeholder",
        "placeholder
        \t\x1b[1;35malias: gphd\x1b[0m",
    );

    // actual printing of commands

    for (k, v) in meta_commands {
        println!("\x1b[1;31m{k}\x1b[0m =>\t{v}");
    }

    println!("\x1b[32m=== print commands ===");
    for (k, v) in print_commands {
        println!("\x1b[1;31m{k}\x1b[0m =>\t{v}");
    }

    println!("\x1b[32m=== graph commands ===");
    for (k, v) in graph_commands {
        println!("\x1b[1;31m{k}\x1b[0m =>\t{v}");
    }
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
