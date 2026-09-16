use crate::commands::command::StdReturn;
use crate::commands::parser::{Out, Redirect};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

pub fn handle(mut std_ret: StdReturn, redirects: &[Redirect], interactive: bool) {
    for redirect in redirects {
        match redirect {
            Redirect::File { out, file, append } => match out {
                Out::StdOut => {
                    let out_string = std_ret.std_out_string.take().unwrap_or_default();
                    write_to_file(PathBuf::from(file), &out_string, *append);
                }
                Out::StdErr => {
                    let err_string = std_ret.std_err_string.take().unwrap_or_default();
                    write_to_file(PathBuf::from(file), &err_string, *append);
                }
            },
        }
    }
    if let Some(out_string) = std_ret.std_out_string {
        print_stream(&out_string, interactive, false);
    }
    if let Some(err_string) = std_ret.std_err_string {
        print_stream(&err_string, interactive, true);
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
