mod commands;

use crate::commands::command::Command;
use std::io::{self, Write};

fn main() {
    loop {
        run();
    }
}

fn run() {
    print!("$ ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Wuh oh!");
    let input = input.trim();

    Command::execute(&Command::from_input(input));
    io::stdout().flush().unwrap();
}
