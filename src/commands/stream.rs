use crate::commands::{
    outcome::Outcome,
    parser::{Out, Redirect},
};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::process::exit;

pub fn handle(outcome: Outcome, redirects: &[Redirect], interactive: bool) {
    let (mut stdout, mut stderr) = match outcome {
        Outcome::Quit(code) => exit(code),
        Outcome::Output { stdout, stderr } => (stdout, stderr),
        Outcome::Ok => (String::new(), String::new()),
    };

    for redirect in redirects {
        match redirect {
            Redirect::File { out, file, append } => match out {
                Out::StdOut => {
                    write_to_file(PathBuf::from(file), &stdout, *append);
                    stdout.clear();
                }
                Out::StdErr => {
                    write_to_file(PathBuf::from(file), &stderr, *append);
                    stderr.clear();
                }
            },
        }
    }
    if !stdout.is_empty() {
        print_stream(&stdout, interactive, false);
    }
    if !stderr.is_empty() {
        print_stream(&stderr, interactive, true);
    }
}

fn print_stream(text: &str, interactive: bool, to_stderr: bool) {
    if interactive {
        let text = text.replace('\n', "\r\n");
        if to_stderr {
            eprint!("{text}\r\n");
        } else {
            print!("{text}\r\n");
        }
    } else if to_stderr {
        eprintln!("{text}");
    } else {
        println!("{text}");
    }
}

fn write_to_file(path: PathBuf, content: &str, append: bool) {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(append)
        .truncate(!append)
        .open(path)
        .unwrap();

    if content.is_empty() {
        return;
    }

    if writeln!(&mut file, "{content}").is_err() {
        eprintln!("Wuh oh!");
    }
}
