use crate::commands::parser::{Out, Redirect};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::exit;
use crate::commands::command::StdReturn;

pub fn handle(mut std_ret: StdReturn, redirects: &[Redirect]) {
    for redirect in redirects {
        match redirect {
            Redirect::File { out, file, append } => {
                match out {
                    Out::StdOut => {
                        let out_string = std_ret.std_out_string.take().unwrap_or_default();
                        write_to_file(PathBuf::from(file), &out_string, *append);
                    }
                    Out::StdErr => {
                        let err_string = std_ret.std_err_string.take().unwrap_or_default();
                        write_to_file(PathBuf::from(file), &err_string, *append);
                    }
                }
            }
            Redirect::None { .. } => {}
        }
    }
    if let Some(out_string) = std_ret.std_out_string {
        println!("{out_string}");
    }
    if let Some(err_string) = std_ret.std_err_string {
        eprintln!("{err_string}");
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

    if let Err(e) = writeln!(&mut file, "{}", content) {
        eprintln!("Wuh oh!");
    }
}