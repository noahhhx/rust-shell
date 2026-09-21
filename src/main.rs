mod commands;
pub mod completion;
pub mod editor;
pub mod shell;

use crate::commands::command::Statement;
use crate::commands::execute;
use crate::commands::outcome::Outcome;
use crate::commands::stream::{self};
use crate::editor::LineEditor;
use crate::shell::Shell;
use std::io::{self, IsTerminal, Write};
use termion::input::TermRead;
use termion::raw::IntoRawMode;

fn main() {
    let interactive = io::stdin().is_terminal() && io::stdout().is_terminal();

    let mut out: Box<dyn Write> = if interactive {
        Box::new(io::stdout().into_raw_mode().unwrap())
    } else {
        Box::new(io::stdout())
    };

    let mut keys = io::stdin().keys();
    let mut shell: Shell = Shell::default();

    loop {
        let mut editor = LineEditor::new(&shell, interactive);
        let Some(line) = editor.read_line(&mut keys, &mut out) else {
            break;
        };

        let Some(stmt) = Statement::parse(&line) else {
            continue;
        };
        match execute(&mut shell, &stmt) {
            Outcome::Quit(_) => break,
            outcome => stream::handle(outcome, &stmt.redirects, interactive),
        }
    }
}
