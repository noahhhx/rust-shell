extern crate core;

mod commands;

use crate::commands::command::Command;
use std::io::{self, Write};
use crate::commands::stream::handle;

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

    let parsed = &Command::parse_line(input);
    if let Some(statement) = parsed {
        let std_ret = Command::execute(statement.command());
        handle(std_ret, &statement.redirects);
    }
    io::stdout().flush().unwrap();
}
