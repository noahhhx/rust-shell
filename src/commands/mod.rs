use std::any::Any;
use std::process::Command;

use crate::commands::builtins::find_in_path;
use crate::commands::{
    builtins::BUILTINS,
    command::Statement,
    outcome::Outcome::{self},
};
use crate::shell::Shell;

pub mod builtins;
pub mod command;
pub mod jobs;
pub mod outcome;
pub mod parser;
pub mod stream;

pub fn execute(shell: &mut Shell, stmt: &Statement) -> Outcome {
    let outcome = match BUILTINS.iter().find(|b| b.name == stmt.program) {
        Some(b) => (b.run)(shell, &stmt.args),
        None => external(stmt),
    };
    shell.last_status = outcome.status();
    outcome
}

fn external(stmt: &Statement) -> Outcome {
    let Some(exe) = find_in_path(&stmt.program) else {
        return Outcome::from_err(format!("{}: command not found", stmt.program));
    };

    let last_arg = stmt.args.last().unwrap_or(&String::from("")).to_owned();
    if last_arg == "&" {
        let pid = Command::new(exe.file_name().unwrap())
            .args(&stmt.args[0..stmt.args.len() - 1])
            .spawn()
            .expect("failed to execute process")
            .id();
        return Outcome::Output {
            stdout: String::from(format!("[1] {}", pid)),
            stderr: String::from(""),
        };
    };

    let output = Command::new(exe.file_name().unwrap())
        .args(&stmt.args)
        .output()
        .expect("failed to execute process");

    let err = String::from_utf8_lossy(output.stderr.trim_ascii_end());
    let err = if err.is_empty() {
        String::new()
    } else {
        format!("{err}")
    };

    let out = String::from_utf8_lossy(output.stdout.trim_ascii_end());
    let out = if out.is_empty() {
        String::new()
    } else {
        out.into_owned()
    };
    Outcome::Output {
        stdout: out,
        stderr: err,
    }
}
