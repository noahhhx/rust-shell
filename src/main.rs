mod commands;
pub mod completion;

use crate::commands::command::Command;
use crate::commands::stream::handle;
use crate::completion::{common_prefix, complete};
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

    while let Some(line) = read_line(&mut keys, stdout.as_mut(), interactive) {
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

    let mut pending_tab: Option<Vec<String>> = None;
    let mut line = String::new();
    while let Some(Ok(key)) = keys.next() {
        if !matches!(key, Key::Char('\t')) {
            pending_tab = None;
        }
        match key {
            Key::Char('\n') => {
                new_line(stdout, interactive);
                return Some(line);
            }
            Key::Char('\t') => {
                pending_tab = tab_complete(pending_tab, stdout, &mut line, interactive);
                stdout.flush().unwrap();
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
            Key::Ctrl('c' | 'd') => {
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

fn tab_complete(
    pending_tab: Option<Vec<String>>,
    stdout: &mut dyn Write,
    line: &mut String,
    interactive: bool,
) -> Option<Vec<String>> {
    if let Some(list) = &pending_tab {
        new_line(stdout, interactive);
        write!(stdout, "{}", list.join("  ")).unwrap();
        new_line(stdout, interactive);
        write!(stdout, "\r$ {line}").unwrap();
        pending_tab
    } else {
        let word_start = line.rfind(char::is_whitespace).map_or(0, |i| i + 1);
        let word = line[word_start..].to_string();
        let candidates = complete(&word, word_start == 0);
        let prefix = common_prefix(&candidates);

        if candidates.len() == 1 {
            line.truncate(word_start);
            line.push_str(&candidates[0]);
            line.push(' ');
            write!(stdout, "\r$ {line}{}", termion::clear::UntilNewline).unwrap();
            None
        } else if prefix.len() > word.len() {
            line.truncate(word_start);
            line.push_str(&prefix);
            if candidates.len() > 1 {
                write!(stdout, "\x07").unwrap();
            }
            write!(stdout, "\r$ {line}{}", termion::clear::UntilNewline).unwrap();
            Some(candidates)
        } else if candidates.len() > 1 {
            write!(stdout, "\x07").unwrap();
            Some(candidates)
        } else {
            write!(stdout, "\x07").unwrap();
            None
        }
    }
}
