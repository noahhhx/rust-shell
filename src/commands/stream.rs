use crate::commands::parser::{Out, Redirect};
use std::fs::OpenOptions;
use std::path::PathBuf;
use crate::commands::command::StdReturn;

pub fn handle(std_ret: Option<StdReturn>, redirect: &[Redirect]) {
    match redirect[0].out() {
        Out::StdOut => {
            if let Some(test) = std_ret {
                match test {
                    StdReturn::StdOut { out_string } => {
                        println!("{out_string}");
                    }
                    StdReturn::StdErr { .. } => {}
                }
            }
        }
        Out::StdErr => {

        }
    }
}

fn write_to_file(path: PathBuf, content: &str, append: bool) {
    let file = OpenOptions::new()
        .write(true)
        .append(append)
        .open(path)
        .unwrap();

    // if let Err(e) = writeln!(file, "{}", content) {
    //     eprintln!("Wuh oh!");
    // }
}