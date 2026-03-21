extern crate rustyline;

use crate::parser::parser::*;
use rustyline::{DefaultEditor, error::ReadlineError};
use std::env;
use std::process;

mod definitions;
mod parser;
mod syntax;

fn main() {
    let args: Vec<String> = env::args().collect();

    // REPL 模式
    if args.len() == 1 {
        run_repl();
    } else {
        eprintln!("Too many arguments!");
    }
}

fn run_repl() {
    let mut rl = DefaultEditor::new().unwrap();

    loop {
        match rl.readline(">>> ") {
            Ok(line) => {
                let _ = rl.add_history_entry(&line);

                if line.starts_with(':') {
                    // :command expression
                    run_repl_command(line);
                }
            }
            Err(ReadlineError::Interrupted) => {
                print!("^C");
                break;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(_) => {
                eprintln!("Error reading line");
                break;
            }
        }
    }
}

fn run_repl_command(line: String) {
    let Some(space) = line.find(' ') else {
        return;
    };
    let (command, expression) = line.split_at(space);

    match command {
        ":quit" | ":q" => {
            process::exit(0);
        }
        ":parse" => {
            let mut parser = Parser::new(expression.trim());
            println!("{:?}", parser.parse_program());
        }
        _ => {
            eprintln!("Unknown command: {}", command);
        }
    }
}
