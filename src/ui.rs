//! Module responsible for handling the CLI

use crate::types::{AspectFull, SongEntries};
use crate::LOCATION_TZ;

use std::collections::HashMap;
use std::error::Error;

use chrono::{DateTime, TimeZone};
use chrono_tz::Tz;
use rustyline::{error::ReadlineError, ColorMode, Config, Editor};

/// `println!("Invalid date! Make sure you input the date in YYYY-MM-DD format.")`
#[macro_export]
macro_rules! inv_date {
    () => {
        println!("Invalid date! Make sure you input the date in YYYY-MM-DD format.")
    };
}

/// For [`ReadlineError`]s
///
/// `println!("Something went wrong! Please try again. Error code: {}", e)`
///
/// Used when matching [`Editor::readline`] ([Result<String, `ReadlineError`>])
/// and printing the error message
///
/// # Examples
///
/// ```
/// let mut rl = Editor::<()>::new();
/// let line = rl.readline(">>> ");
/// match line {
///    Ok(usr_input) => (),
///    Err(e) => rle!(e),
///}
/// ```
#[macro_export]
macro_rules! rle {
    ($error:ident) => {
        println!(
            "Something went wrong! Please try again. Error code: {}",
            $error
        )
    };
}

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
            Ok(usr_input) => match_input(&usr_input, entries, &mut rl),
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
fn match_input(inp: &str, entries: &SongEntries, rl: &mut Editor<()>) {
    let inp = inp;
    match inp {
        "help" | "h" => help(),
        "print artist" | "part" => match match_print_artist(entries, rl) {
            Ok(_) => (),
            Err(e) => println!("{e}"),
        },
        "print album" | "palb" => match_print_album(entries, rl),
        "print song" | "pson" => match_print_song(entries, rl),
        "print songs" | "psons" => match_print_songs(entries, rl),
        "print artist date" | "partd" => match_print_artist_date(entries, rl),
        "print album date" | "palbd" => match_print_album_date(entries, rl),
        "print song date" | "psond" => match_print_song_date(entries, rl),
        "print songs date" | "psonsd" => match_print_songs_date(entries, rl),
        // when you press ENTER -> nothing happens, new prompt
        "" => (),
        _ => {
            // \x1b[1;31m makes text red
            // \x1b[0m makes it the default color
            println!("Command not found! Type \x1b[1;31mhelp\x1b[0m to print available commands");
        }
    }
}

/// Used by [`match_input`()] for `print artist` command
fn match_print_artist(entries: &SongEntries, rl: &mut Editor<()>) -> Result<(), Box<dyn Error>> {
    // prompt: artist name
    println!("Artist name?");
    let usr_input_art = match rl.readline(PROMPT_MAIN) {
        Ok(usr_input) => usr_input,
        Err(e) => return Err(Box::new(e)),
    };

    let art = match entries.find().artist(&usr_input_art) {
        Ok(art) => art,
        Err(e) => return Err(Box::new(e)),
    };

    entries.print_aspect(&AspectFull::Artist(&art));
    Ok(())
}

/// Used by [`match_input`()] for `print artist date` command
///
/// Basically [`match_print_artist`()] but with date functionality
fn match_print_artist_date(entries: &SongEntries, rl: &mut Editor<()>) {
    // 1st prompt: artist name
    println!("Artist name?");
    let line_art = rl.readline(PROMPT_MAIN);
    match line_art {
        Ok(usr_input) => match entries.find().artist(&usr_input) {
            Ok(art) => {
                // 2nd prompt: start date
                println!("Start date? YYYY-MM-DD");
                let line_start_date = rl.readline(PROMPT_SECONDARY);
                match line_start_date {
                    Ok(usr_input) => match user_input_date_parser(&usr_input) {
                        Ok(start_date) => {
                            // 3rd prompt: end date
                            println!("End date? YYYY-MM-DD");
                            let line_end_date = rl.readline(PROMPT_SECONDARY);
                            match line_end_date {
                                Ok(usr_input) => match user_input_date_parser(&usr_input) {
                                    Ok(end_date) => entries.print_aspect_date(
                                        &AspectFull::Artist(&art),
                                        &start_date,
                                        &end_date,
                                    ),
                                    Err(_) => inv_date!(),
                                },
                                Err(e) => rle!(e),
                            }
                        }
                        Err(_) => inv_date!(),
                    },
                    Err(e) => rle!(e),
                }
            }
            Err(e) => println!("{e}"),
        },
        Err(e) => rle!(e),
    }
}

/// Used by [`match_input`()] for `print album` command
fn match_print_album(entries: &SongEntries, rl: &mut Editor<()>) {
    // 1st prompt: artist name
    println!("Artist name?");
    let line_art = rl.readline(PROMPT_MAIN);
    match line_art {
        Ok(usr_input_art) => match entries.find().artist(&usr_input_art) {
            Ok(art) => {
                // 2nd prompt: album name
                println!("Album name?");
                let line_alb = rl.readline(PROMPT_MAIN);
                match line_alb {
                    Ok(usr_input_alb) => match entries.find().album(&usr_input_alb, &art.name) {
                        Ok(alb) => entries.print_aspect(&AspectFull::Album(&alb)),
                        Err(e) => println!("{e}"),
                    },
                    Err(e) => rle!(e),
                }
            }
            Err(e) => println!("{e}"),
        },
        Err(e) => rle!(e),
    }
}

/// Used by [`match_input`()] for `print album date` command
///
/// Basically [`match_print_album`()] but with date functionality
fn match_print_album_date(entries: &SongEntries, rl: &mut Editor<()>) {
    // 1st prompt: artist name
    println!("Artist name?");
    let line_art = rl.readline(PROMPT_MAIN);
    match line_art {
        Ok(usr_input_art) => match entries.find().artist(&usr_input_art) {
            Ok(art) => {
                // 2nd prompt: album name
                println!("Album name?");
                let line_alb = rl.readline(PROMPT_MAIN);
                match line_alb {
                    Ok(usr_input_alb) => match entries.find().album(&usr_input_alb, &art.name) {
                        Ok(alb) => {
                            // 3rd prompt: start date
                            println!("Start date? YYYY-MM-DD");
                            let line_start_date = rl.readline(PROMPT_SECONDARY);
                            match line_start_date {
                                Ok(usr_input) => match user_input_date_parser(&usr_input) {
                                    Ok(start_date) => {
                                        // 4th prompt: end date
                                        println!("End date? YYYY-MM-DD");
                                        let line_end_date = rl.readline(PROMPT_SECONDARY);
                                        match line_end_date {
                                            Ok(usr_input) => {
                                                match user_input_date_parser(&usr_input) {
                                                    Ok(end_date) => entries.print_aspect_date(
                                                        &AspectFull::Album(&alb),
                                                        &start_date,
                                                        &end_date,
                                                    ),
                                                    Err(_) => inv_date!(),
                                                }
                                            }
                                            Err(e) => rle!(e),
                                        }
                                    }
                                    Err(_) => inv_date!(),
                                },
                                Err(e) => rle!(e),
                            }
                        }
                        Err(e) => println!("{e}"),
                    },
                    Err(e) => rle!(e),
                }
            }
            Err(e) => println!("{e}"),
        },
        Err(e) => rle!(e),
    }
}

/// Used by [`match_input`()] for `print song` command
fn match_print_song(entries: &SongEntries, rl: &mut Editor<()>) {
    // 1st prompt: artist name
    println!("Artist name?");
    let line_art = rl.readline(PROMPT_MAIN);
    match line_art {
        Ok(usr_input_art) => {
            match entries.find().artist(&usr_input_art) {
                Ok(art) => {
                    // 2nd prompt: album name
                    println!("Album name?");
                    let line_alb = rl.readline(PROMPT_MAIN);
                    match line_alb {
                        Ok(usr_input_alb) => {
                            match entries.find().album(&usr_input_alb, &art.name) {
                                Ok(alb) => {
                                    // 3rd prompt: song name
                                    println!("Song name?");
                                    let line_son = rl.readline(PROMPT_MAIN);
                                    match line_son {
                                        Ok(usr_input_son) => match entries.find().song_from_album(
                                            &usr_input_son,
                                            &alb.name,
                                            &alb.artist.name,
                                        ) {
                                            Ok(son) => {
                                                entries.print_aspect(&AspectFull::Song(&son));
                                            }
                                            Err(e) => println!("{e}"),
                                        },
                                        Err(e) => rle!(e),
                                    }
                                }
                                Err(e) => println!("{e}"),
                            }
                        }
                        Err(e) => rle!(e),
                    }
                }
                Err(e) => println!("{e}"),
            }
        }
        Err(e) => rle!(e),
    }
}

/// Used by [`match_input`()] for `print song date` command
///
/// Basically [`match_print_song`()] but with date functionality
fn match_print_song_date(entries: &SongEntries, rl: &mut Editor<()>) {
    // 1st prompt: artist name
    println!("Artist name?");
    let line_art = rl.readline(PROMPT_MAIN);
    match line_art {
        Ok(usr_input_art) => {
            match entries.find().artist(&usr_input_art) {
                Ok(art) => {
                    // 2nd prompt: album name
                    println!("Album name?");
                    let line_alb = rl.readline(PROMPT_MAIN);
                    match line_alb {
                        Ok(usr_input_alb) => {
                            match entries.find().album(&usr_input_alb, &art.name) {
                                Ok(alb) => {
                                    // 3rd prompt: song name
                                    println!("Song name?");
                                    let line_son = rl.readline(PROMPT_MAIN);
                                    match line_son {
                                        Ok(usr_input_son) => match entries.find().song_from_album(
                                            &usr_input_son,
                                            &alb.name,
                                            &alb.artist.name,
                                        ) {
                                            Ok(son) => {
                                                // 4th prompt: start date
                                                println!("Start date? YYYY-MM-DD");
                                                let line_start_date = rl.readline(PROMPT_SECONDARY);
                                                match line_start_date {
                                                    Ok(usr_input) => {
                                                        match user_input_date_parser(&usr_input) {
                                                            Ok(start_date) => {
                                                                // 5th prompt: end date
                                                                println!("End date? YYYY-MM-DD");
                                                                let line_end_date =
                                                                    rl.readline(PROMPT_SECONDARY);
                                                                match line_end_date {
                                                                    Ok(usr_input) => {
                                                                        match user_input_date_parser(
                                                                            &usr_input,
                                                                        ) {
                                                                            Ok(end_date) => {
                                                                                entries.print_aspect_date(&AspectFull::Song(&son), &start_date, &end_date);
                                                                            }
                                                                            Err(_) => inv_date!(),
                                                                        }
                                                                    }
                                                                    Err(e) => rle!(e),
                                                                }
                                                            }
                                                            Err(_) => inv_date!(),
                                                        }
                                                    }
                                                    Err(e) => rle!(e),
                                                }
                                            }
                                            Err(e) => println!("{e}"),
                                        },
                                        Err(e) => rle!(e),
                                    }
                                }
                                Err(e) => println!("{e}"),
                            }
                        }
                        Err(e) => rle!(e),
                    }
                }
                Err(e) => println!("{e}"),
            }
        }
        Err(e) => rle!(e),
    }
}

/// Used by [`match_input`()] for `print songs` command
fn match_print_songs(entries: &SongEntries, rl: &mut Editor<()>) {
    // 1st prompt: artist name
    println!("Artist name?");
    let line_art = rl.readline(PROMPT_MAIN);
    match line_art {
        Ok(usr_input_art) => {
            match entries.find().artist(&usr_input_art) {
                Ok(art) => {
                    // 2nd prompt: song name
                    println!("Song name?");
                    let line_son = rl.readline(PROMPT_MAIN);
                    match line_son {
                        Ok(usr_input_son) => match entries.find().song(usr_input_son, art.name) {
                            Ok(songs) => {
                                if songs.len() == 1 {
                                    entries.print_aspect(&AspectFull::Song(&songs[0]));
                                } else {
                                    println!(
                                        "I've found {} songs named {} from {}!",
                                        songs.len(),
                                        &songs[0].name,
                                        &songs[0].album.artist.name
                                    );
                                    for song in songs {
                                        entries.print_aspect(&AspectFull::Song(&song));
                                    }
                                }
                            }
                            Err(e) => println!("{e}"),
                        },
                        Err(e) => rle!(e),
                    }
                }
                Err(e) => println!("{e}"),
            }
        }
        Err(e) => rle!(e),
    }
}

/// Used by [`match_input`()] for `print songs date` command
fn match_print_songs_date(entries: &SongEntries, rl: &mut Editor<()>) {
    // 1st prompt: artist name
    println!("Artist name?");
    let line_art = rl.readline(PROMPT_MAIN);
    match line_art {
        Ok(usr_input_art) => {
            match entries.find().artist(&usr_input_art) {
                Ok(art) => {
                    // 2nd prompt: song name
                    println!("Song name?");
                    let line_son = rl.readline(PROMPT_MAIN);
                    match line_son {
                        Ok(usr_input_son) => {
                            match entries.find().song(usr_input_son, art.name) {
                                Ok(songs) => {
                                    // 3rd prompt: start date
                                    println!("Start date? YYYY-MM-DD");
                                    let line_start_date = rl.readline(PROMPT_SECONDARY);
                                    match line_start_date {
                                        Ok(usr_input) => match user_input_date_parser(&usr_input) {
                                            Ok(start_date) => {
                                                // 4th prompt: end date
                                                println!("End date? YYYY-MM-DD");
                                                let line_end_date = rl.readline(PROMPT_SECONDARY);
                                                match line_end_date {
                                                    Ok(usr_input) => {
                                                        match user_input_date_parser(&usr_input) {
                                                            Ok(end_date) => {
                                                                if songs.len() == 1 {
                                                                    entries.print_aspect_date(
                                                                        &AspectFull::Song(
                                                                            &songs[0],
                                                                        ),
                                                                        &start_date,
                                                                        &end_date,
                                                                    );
                                                                } else {
                                                                    println!(
                                                "I've found {} songs named {} from {}!",
                                                songs.len(),
                                                &songs[0].name,
                                                &songs[0].album.artist.name
                                            );
                                                                    for song in songs {
                                                                        entries.print_aspect_date(
                                                                            &AspectFull::Song(
                                                                                &song,
                                                                            ),
                                                                            &start_date,
                                                                            &end_date,
                                                                        );
                                                                    }
                                                                }
                                                            }
                                                            Err(_) => inv_date!(),
                                                        }
                                                    }
                                                    Err(e) => rle!(e),
                                                }
                                            }
                                            Err(_) => inv_date!(),
                                        },
                                        Err(e) => rle!(e),
                                    }
                                }
                                Err(e) => println!("{e}"),
                            }
                        }
                        Err(e) => rle!(e),
                    }
                }
                Err(e) => println!("{e}"),
            }
        }
        Err(e) => rle!(e),
    }
}

/// Used by [`match_input`()] for `help` command
///
/// Prints the available commands to the [`std::io::stdout`]
fn help() {
    let mut commands: HashMap<&str, &str> = HashMap::new();
    // alias in pink! \x1b[1;35m
    commands.insert(
        "help",
        "prints this command list
        \t\x1b[1;35malias: h",
    );

    commands.insert(
        "print artist",
        "prints every album from the artist
        \topens another prompt where you input the artist name
        \t\x1b[1;35malias: part",
    );
    commands.insert(
        "print album",
        "prints every song from the album
        \topens another prompt where you input the artist name
        \tand then the album name
        \t\x1b[1;35malias: palb",
    );
    commands.insert(
        "print song",
        "prints a song
        \topens another prompt where you input the artist name
        \tand then the album name
        \tand then the song name
        \t\x1b[1;35malias: pson",
    );
    commands.insert(
        "print songs",
        "prints a song with all the albums it may be from
        \topens another prompt where you input the artist name
        \tand then the song name
        \t\x1b[1;35malias: psons",
    );

    commands.insert(
        "print artist date",
        "prints every album from the artist within a date range
        \topens another prompt where you input the artist name
        \tand then the date range
        \t\x1b[1;35malias: partd",
    );
    commands.insert(
        "print album date",
        "prints every song from the album within a date range
        \topens another prompt where you input the artist name
        \tand then the album name
        \t\x1b[1;35malias: palbd",
    );
    commands.insert(
        "print song date",
        "prints a song within a date range
        \topens another prompt where you input the artist name
        \tand then the album name
        \tand then the song name
        \tand then the date range
        \t\x1b[1;35malias: psond",
    );
    commands.insert(
        "print songs date",
        "prints a song with all the albums it may be from within a date range
        \topens another prompt where you input the artist name
        \tand then the song name
        \tand then the date range
        \t\x1b[1;35malias: psonsd",
    );

    for (k, v) in commands {
        // makes the command itself red and the rest default color
        println!("\x1b[1;31m{k}\x1b[0m =>\t{v}");
    }
}

/// used by `*_date` functions in this module for when the user inputs a date
fn user_input_date_parser(usr_input: &str) -> Result<DateTime<Tz>, chrono::format::ParseError> {
    // usr_input should be in YYYY-MM-DD format
    let date_str = format!("{usr_input}T00:00:00Z");

    // "%FT%TZ" is equivalent to "%Y-%m-%dT%H:%M:%SZ"
    // see <https://docs.rs/chrono/latest/chrono/format/strftime/index.html>
    LOCATION_TZ.datetime_from_str(&date_str, "%FT%TZ")
}
