extern crate core;

mod commands;

use crate::commands::command::{BUILT_IN_COMMANDS, Command, find_in_path_starts_with};
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
                if let Some(list) = &pending_tab {
                    new_line(stdout, interactive);
                    for item in list {
                        write!(stdout, "{item}  ").unwrap();
                    }
                    new_line(stdout, interactive);
                    write!(stdout, "\r$ {line}").unwrap();
                } else {
                    let cands = tab_complete(stdout, &mut line);
                    pending_tab = (cands.len() > 1).then_some(cands.clone());
                    if cands.len() == 1 {
                        line = cands[0].clone() + " ";
                        write!(stdout, "\r$ {line}").unwrap();
                    }
                }
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

fn tab_complete(stdout: &mut dyn Write, line: &mut String) -> Vec<String> {
    let typed = line.clone();
    let mut candidates: Vec<String> = Vec::new();

    for command in BUILT_IN_COMMANDS {
        if command.starts_with(&typed) {
            candidates.push(command.to_string());
        }
    }

    for entry in find_in_path_starts_with(&typed) {
        if !candidates.contains(&entry) {
            candidates.push(entry);
        }
    }

    if let Some(ret) = common_longest_prefix(&typed, &candidates) {
        *line = ret;
        write!(stdout, "\r$ {line}").unwrap();
        return Vec::new();
    }

    if candidates.is_empty() || candidates.len() > 1 {
        line.push('\x07');
        write!(stdout, "\x07").unwrap();
    }
    candidates.sort();
    candidates
}

fn common_longest_prefix(typed: &str, candidates: &Vec<String>) -> Option<String> {
    let mut count = 0;
    let mut current = String::new();
    for cand in candidates {
        if cand.starts_with(typed)
            && !cand.eq(typed)
            && (count == 0 || (count > 0 && current.len() > cand.len()))
        {
            current.clone_from(cand);
            count += 1;
        }
    }
    if count > 1 {
        return Some(current);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_longest_prefix() {
        let typed = "xyz_";
        let candidates: Vec<String> = vec![
            "xyz_foo".to_string(),
            "xyz_foo_bar".to_string(),
            "xyz_foo_bar_baz".to_string(),
        ];

        assert_eq!(
            "xyz_foo",
            common_longest_prefix(typed, &candidates).unwrap()
        )
    }
}
