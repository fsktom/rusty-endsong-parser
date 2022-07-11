//! Module responsible for handling the CLI

use crate::types::{AspectFull, SongEntries};

use std::collections::HashMap;

use rustyline::{error::ReadlineError, ColorMode, Config, Editor};

/// Starts the CLI/shell instance
pub fn start(entries: &SongEntries) {
    // I SWEAR TO GOD THIS IS ACCIDENTAL THAT THE STRINGS ARE THE SAME LENGTH
    // WTF?!? HOW AM I SO LUCKY XDD
    // I DIDN'T NOTICE THIS UNTIL 20mins LATER
    println!("=== INTERACTIVE MODE ACTIVATED ===");
    println!("PRESS 'CTRL+C' TO EXIT THE PROGRAM");
    println!("TYPE 'help' FOR AVAILABLE COMMANDS");
    // https://old.reddit.com/r/rust/comments/vrdmuf/introducing_my_first_rust_project_trane_an/
    // => https://github.com/trane-project/trane-cli
    // ==> https://github.com/kkawakam/rustyline

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
        "help" => help(),
        "print artist" => {
            // makes the prompt cyan and the rest default color
            let line = rl.readline("  \x1b[1;36m>>\x1b[0m ");
            match line {
                Ok(usr_input) => {
                    let art_opt = entries.find().artist(usr_input);
                    match art_opt {
                        Some(art) => entries.print_aspect(AspectFull::Artist(&art)),
                        None => {
                            println!("Sorry, I couldn't find any artist with that name!")
                        }
                    }
                }
                Err(e) => println!("Something went wrong! Please try again. Error code: {}", e),
            }
        }
        _ => (),
    }
}

/// Prints the available commands to the [std::io::stdout]
fn help() {
    let mut commands: HashMap<&str, &str> = HashMap::new();
    commands.insert("help", "prints this command list");
    commands.insert(
        "print artist",
        "prints every album from the artist -
        \topens another prompt where you input the artist name",
    );

    for (k, v) in commands {
        // makes the command itself red and the rest default color
        println!("\x1b[1;31m{}\x1b[0m => {}", k, v)
    }
}
