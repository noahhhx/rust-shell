use std::{
    io::{self, Write},
    mem::take,
};

use termion::{event::Key, input::Keys};

use crate::{
    completion::{self, CompletionAction},
    shell::Shell,
};

pub enum TabState {
    Idle,
    Ambiguous(Vec<String>),
}

pub enum EditResult {
    StillEditing,
    Submitted(String),
    Cancelled, // ctrl+c ctrl+d
}

pub struct LineEditor<'a> {
    line: String,
    cursor: usize,
    tab: TabState,
    shell: &'a Shell,
    interactive: bool,
}

impl<'a> LineEditor<'a> {
    #[must_use]
    pub fn new(shell: &'a Shell, interactive: bool) -> Self {
        Self {
            line: String::new(),
            cursor: 0,
            tab: TabState::Idle,
            shell,
            interactive,
        }
    }

    pub fn read_line(&mut self, keys: &mut Keys<io::Stdin>, out: &mut dyn Write) -> Option<String> {
        self.render(out);

        while let Some(Ok(key)) = keys.next() {
            match self.feed_key(key, out) {
                EditResult::StillEditing => {}
                EditResult::Submitted(line) => return Some(line),
                EditResult::Cancelled => {
                    // parity with today's main.rs:82-87 — REFACTORING.md flag #3, pick deliberately
                    if self.line.is_empty() {
                        return None;
                    }
                    self.new_line(out);
                    return Some(take(&mut self.line));
                }
            }
        }
        if self.line.is_empty() {
            None
        } else {
            Some(take(&mut self.line))
        }
    }

    fn feed_key(&mut self, key: Key, out: &mut dyn Write) -> EditResult {
        if !matches!(key, Key::Char('\t')) {
            self.tab = TabState::Idle;
        }
        match key {
            Key::Char('\n') => {
                self.new_line(out);
                EditResult::Submitted(take(&mut self.line))
            }
            Key::Char('\t') => {
                if let TabState::Ambiguous(list) = std::mem::replace(&mut self.tab, TabState::Idle)
                // consume held state
                {
                    // consecutive Tab:
                    self.new_line(out);
                    write!(out, "{}", list.join("  ")).unwrap();
                    self.new_line(out);
                    self.render(out);
                } else {
                    let action = completion::complete(self.shell, &self.line);
                    let bell = matches!(action, CompletionAction::None | CompletionAction::Show(_));
                    if bell {
                        write!(out, "\x07").unwrap();
                    }
                    self.apply(action);
                    self.render(out);
                }
                EditResult::StillEditing
            }
            Key::Char(c) => {
                self.line.push(c);
                self.cursor += 1;
                write!(out, "{c}").unwrap();
                out.flush().unwrap();
                EditResult::StillEditing
            }
            Key::Backspace => {
                if self.line.pop().is_some() {
                    self.cursor -= 1;
                    if self.interactive {
                        self.render(out);
                    } else {
                        write!(out, "\u{8} \u{8}").unwrap();
                        out.flush().unwrap();
                    }
                }
                EditResult::StillEditing
            }
            Key::Ctrl('c' | 'd') => EditResult::Cancelled,
            _ => EditResult::StillEditing,
        }
    }

    fn apply(&mut self, action: CompletionAction) {
        match action {
            CompletionAction::Replace { start, text, space } => {
                self.line.truncate(start);
                self.line.push_str(&text);
                if space && !text.ends_with('/') {
                    self.line.push(' ');
                }
                self.cursor = self.line.len();
                self.tab = TabState::Idle;
            }
            CompletionAction::Show(list) => self.tab = TabState::Ambiguous(list),
            CompletionAction::None => {}
        }
    }

    fn render(&mut self, out: &mut dyn Write) {
        write!(out, "\r$ {}{}", self.line, termion::clear::UntilNewline).unwrap();
        out.flush().unwrap();
    }

    fn new_line(&self, out: &mut dyn Write) {
        if self.interactive {
            write!(out, "\r\n").unwrap();
        } else {
            writeln!(out).unwrap();
        }
        out.flush().unwrap();
    }
}
