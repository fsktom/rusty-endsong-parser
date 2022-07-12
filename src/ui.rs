//! Module responsible for handling the CLI

use crate::types::{AspectFull, SongEntries};

use std::collections::HashMap;

use rustyline::{error::ReadlineError, ColorMode, Config, Editor};

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

    let mut rl = Editor::<()>::with_config(config);

    let history_path = std::path::Path::new(".rep_history");
    if !history_path.exists() {
        match std::fs::File::create(history_path) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to create history file: {}", e);
            }
        }
    }
    match rl.load_history(history_path) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Failed to load history file at .rep_history: {}", e);
        }
    }

    loop {
        // https://bixense.com/clicolors/
        // \x1b[1;32m makes ">>>" green
        // \x1b[0m makes user input default color again
        let line = rl.readline("\x1b[1;32m>>>\x1b[0m ");
        match line {
            Ok(usr_input) => match_input(usr_input, entries, &mut rl),
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    match rl.save_history(history_path) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Failed to save history to file .trane_history: {}", e);
        }
    }
}

/// Decides what to do with user input
fn match_input(inp: String, entries: &SongEntries, rl: &mut Editor<()>) {
    let inp = inp.as_str();
    match inp {
        "help" | "h" => help(),
        "print artist" | "part" => {
            println!("Artist name?");
            // makes the prompt cyan and the rest default color
            let line = rl.readline("  \x1b[1;36m>>\x1b[0m ");
            match line {
                Ok(usr_input) => match entries.find().artist(usr_input) {
                    Ok(art) => {
                        entries.print_aspect(AspectFull::Artist(&art));
                    }
                    Err(e) => println!("{}", e),
                },
                Err(e) => println!("Something went wrong! Please try again. Error code: {}", e),
            }
        }
        "print album" | "palb" => {
            println!("Artist name?");
            // makes the prompt cyan and the rest default color
            let line = rl.readline("  \x1b[1;36m>>\x1b[0m ");
            match line {
                Ok(usr_input_art) => match entries.find().artist(usr_input_art) {
                    Ok(art) => {
                        println!("Album name?");
                        let line_alb = rl.readline("  \x1b[1;36m>>\x1b[0m ");

                        match line_alb {
                            Ok(usr_input_alb) => {
                                match entries.find().album(usr_input_alb, art.name) {
                                    Ok(alb) => {
                                        entries.print_aspect(AspectFull::Album(&alb));
                                    }
                                    Err(e) => println!("{}", e),
                                }
                            }
                            Err(e) => {
                                println!(
                                    "Something went wrong! Please try again. Error code: {}",
                                    e
                                )
                            }
                        }
                    }
                    Err(e) => println!("{}", e),
                },
                Err(e) => println!("Something went wrong! Please try again. Error code: {}", e),
            }
        }
        "" => (),
        _ => {
            // \x1b[1;31m makes text red
            // \x1b[0m makes it the default color
            println!("Command not found! Type \x1b[1;31mhelp\x1b[0m to print available commands")
        }
    }
}

/// Prints the available commands to the [std::io::stdout]
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
        "prints every song from the album -
        \topens another prompt where you input the artist name
        \tand then the album name
        \t\x1b[1;35malias: palb",
    );

    for (k, v) in commands {
        // makes the command itself red and the rest default color
        println!("\x1b[1;31m{}\x1b[0m => {}", k, v)
    }
}
