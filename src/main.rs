#[allow(unused_imports)]
use std::io::{self, Write};
use std::iter::once;
use std::process::exit;

fn main() {
    loop {
        run();
    }
}

fn run() {
    print!("$ ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Wuh oh!");
    let input = input.trim();

    handle_command(input);
    io::stdout().flush().unwrap();
}

fn handle_command(input: &str) {
    let (command, args) = match input.split_once(' ') {
        None => {
            (input, "")
        }
        Some((first, rest)) => {
            (first, rest)
        }
    };

    match command {
        "exit" => {
            exit(0);
        }
        "echo" => {
            println!("{args}")
        }
        _ => {
            println!("{}: command not found", input);
        }
    }
}
