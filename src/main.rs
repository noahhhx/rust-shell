extern crate core;

mod commands;

use crate::commands::command::Command;
use crate::commands::stream::handle;
use std::io::{self, IsTerminal, Write};
use termion::event::Key;
use termion::input::{Keys, TermRead};
use termion::raw::IntoRawMode;

fn main() {
    let interactive = io::stdin().is_terminal() && io::stdout().is_terminal();

    let mut stdout: Box<dyn Write> = if interactive {
        Box::new(io::stdout().into_raw_mode().unwrap())
    } else {
        Box::new(io::stdout())
    };

    let mut keys = io::stdin().keys();

    loop {
        let Some(line) = read_line(&mut keys, stdout.as_mut(), interactive) else {
            break;
        };

        let Some(statement) = Command::parse_line(&line) else {
            continue;
        };
        if matches!(statement.command(), Command::Exit) {
            break;
        }
        if let Some(std_ret) = statement.command.execute() {
            handle(std_ret, &statement.redirects, interactive);
        }
    }
}

fn read_line(
    keys: &mut Keys<io::Stdin>,
    stdout: &mut dyn Write,
    interactive: bool,
) -> Option<String> {
    write!(stdout, "$ ").unwrap();
    stdout.flush().unwrap();

    let mut line = String::new();
    while let Some(Ok(key)) = keys.next() {
        match key {
            Key::Char('\n') => {
                new_line(stdout, interactive);
                return Some(line);
            }
            Key::Char(c) => {
                line.push(c);
                write!(stdout, "{c}").unwrap();
                stdout.flush().unwrap();
            }
            Key::Backspace => {
                if line.pop().is_some() {
                    if interactive {
                        write!(stdout, "\r$ {line}{}", termion::clear::UntilNewline).unwrap();
                    } else {
                        write!(stdout, "\u{8} \u{8}").unwrap();
                    }
                    stdout.flush().unwrap();
                }
            }
            Key::Ctrl('c') | Key::Ctrl('d') => {
                break;
            }
            _ => {}
        }
    }

    if line.is_empty() {
        None
    } else {
        new_line(stdout, interactive);
        Some(line)
    }
}

fn new_line(stdout: &mut dyn Write, interactive: bool) {
    if interactive {
        write!(stdout, "\r\n").unwrap();
    } else {
        writeln!(stdout).unwrap();
    }
    stdout.flush().unwrap();
}
