use crate::commands::parser::{Out, Redirect};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::exit;
use crate::commands::command::StdReturn;

pub fn handle(std_ret: StdReturn, redirects: &[Redirect]) {

    let out_string = match std_ret {
        StdReturn::StdOut { out_string } => {
            out_string
        }
        StdReturn::StdErr { err_string, exit_code } => {
            eprintln!("{err_string}");
            return;
        }
    };

    if let Some(redirect) = redirects.iter().next() {
        match redirect {
            Redirect::File { out, file, append } => {
                write_to_file(PathBuf::from(file), &out_string, *append);
            }
            Redirect::None { .. } => {
                println!("test")
            }
        }
    } else {
        println!("{out_string}");
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

    if let Err(e) = writeln!(&mut file, "{}", content) {
        eprintln!("Wuh oh!");
    }
}