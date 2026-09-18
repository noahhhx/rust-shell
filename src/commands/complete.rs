use crate::commands::command::{StdReturn, default_err};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

static COMPLETION_MAP: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn complete(args: &[String]) -> StdReturn {
    let (flag, args) = parse_complete_args(args);
    match flag.as_str() {
        "-p" => print_completion(&args),
        "-C" => register_completion(&args),
        _ => default_err(),
    }
}

fn register_completion(args: &[String]) -> StdReturn {
    let [first, second, ..] = args else {
        return default_err();
    };
    COMPLETION_MAP
        .lock()
        .expect("completion map lock poisoned")
        .insert(second.clone(), first.clone());
    StdReturn::empty()
}

fn print_completion(arg: &[String]) -> StdReturn {
    if arg.is_empty() {
        StdReturn::from_out("no completion specification".to_string())
    } else {
        let cmd = arg.first().unwrap();
        if let Some(result) = COMPLETION_MAP
            .lock()
            .expect("completion map lock fucked")
            .get(cmd) {
            StdReturn::from_out(format!(
                "complete -C '{result}' {cmd}"
            ))
        } else {
            StdReturn::from_out(format!(
                "complete: {}: no completion specification",
                cmd
            ))
        }
    }
}

fn parse_complete_args(args: &[String]) -> (String, Vec<String>) {
    let mut iter = args.iter();
    if let Some(flag) = iter.next() {
        let remaining: Vec<String> = iter.map(ToString::to_string).collect();
        if remaining.is_empty() {
            (flag.clone(), Vec::new())
        } else {
            (flag.clone(), remaining.clone())
        }
    } else {
        (String::new(), Vec::new())
    }
}
