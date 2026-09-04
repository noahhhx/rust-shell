#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    print!("$ ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Wuh oh!");

    handle_command(input);
    io::stdout().flush().unwrap();
}

fn handle_command(input: String) {
    println!("{input}: command not found");
}
