use std::{
    ffi::OsStr,
    io::Read,
    process::{Command, Stdio},
    thread,
};

use crate::commands::{command::Statement, outcome::Outcome, stream::handle};

pub fn run_job(cmd: &OsStr, stmt: &Statement) -> Outcome {
    let mut child = Command::new(cmd)
        .args(&stmt.args[0..stmt.args.len() - 1])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to execute process");

    let mut stdout = child.stdout.take().unwrap();
    thread::spawn(move || {
        let mut string: String = String::new();
        let _test = stdout.read_to_string(&mut string).unwrap();
        handle(
            Outcome::Output {
                stdout: string.trim().to_string(),
                stderr: String::new(),
            },
            &Vec::new(),
            true,
        );
    });

    Outcome::Output {
        stdout: format!("[1] {}", child.id()),
        stderr: String::new(),
    }
}
