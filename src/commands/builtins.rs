use std::{
    env::{current_dir, set_current_dir},
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use crate::{commands::outcome::Outcome, shell::Shell};

type BuiltinFn = fn(&mut Shell, &[String]) -> Outcome;

pub struct Builtin {
    pub name: &'static str,
    pub run: BuiltinFn,
}

pub const BUILTINS: &[Builtin] = &[
    Builtin {
        name: "echo",
        run: echo,
    },
    Builtin {
        name: "type",
        run: type_cmd,
    },
    Builtin {
        name: "pwd",
        run: pwd,
    },
    Builtin {
        name: "cd",
        run: cd,
    },
    Builtin {
        name: "complete",
        run: complete_cmd,
    },
    Builtin {
        name: "exit",
        run: exit_cmd,
    },
];

fn echo(_shell: &mut Shell, args: &[String]) -> Outcome {
    Outcome::Output {
        stdout: args.join(" "),
        stderr: String::new(),
    }
}

fn type_cmd(_shell: &mut Shell, args: &[String]) -> Outcome {
    let stderr = if args.is_empty() {
        String::from("no arg")
    } else {
        String::new()
    };
    let stdout = if let Some(program) = args.first() {
        if is_builtin(program) {
            format!("{program} is a shell builtin")
        } else if let Some(path) = find_in_path(program) {
            format!("{program} is {}", path.display())
        } else {
            format!("{program}: not found")
        }
    } else {
        String::new()
    };
    Outcome::Output { stdout, stderr }
}

fn pwd(_shell: &mut Shell, _args: &[String]) -> Outcome {
    let (stdout, stderr) = match current_dir() {
        Ok(cur_dir) => (cur_dir.display().to_string(), String::new()),
        Err(_) => (
            String::new(),
            String::from("problem reading current directory"),
        ),
    };
    Outcome::Output { stdout, stderr }
}

fn cd(_shell: &mut Shell, args: &[String]) -> Outcome {
    let stderr = if let Some(path) = args.first() {
        let target = expand_tilde(path);
        if set_current_dir(&target).is_err() {
            format!("cd: {}: No such file or directory", target.display())
        } else {
            return Outcome::Ok;
        }
    } else {
        String::from("Need exactly one argument.")
    };
    Outcome::Output {
        stdout: String::new(),
        stderr,
    }
}

fn complete_cmd(shell: &mut Shell, args: &[String]) -> Outcome {
    let (flag, args) = parse_complete_args(args);
    match flag.as_str() {
        "-p" => print_completion(shell, &args),
        "-C" => register_completion(shell, &args),
        "-r" => remove_completion(shell, &args),
        _ => Outcome::Ok,
    }
}

fn exit_cmd(_shell: &mut Shell, _args: &[String]) -> Outcome {
    Outcome::Quit(0)
}

pub fn is_builtin(program: &str) -> bool {
    BUILTINS.iter().find(|b| b.name == program).is_some()
}

pub fn find_in_path(cmd: &str) -> Option<PathBuf> {
    let path_var = std::env::var("PATH").unwrap_or_default();
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(cmd);
        if candidate.is_file() && is_executable(&candidate) {
            return Some(candidate);
        }
    }
    None
}

pub fn is_executable(path: &Path) -> bool {
    match fs::metadata(path) {
        Ok(metadata) => {
            let mode = metadata.permissions().mode();
            mode & 0o111 != 0
        }
        Err(_) => false,
    }
}

fn expand_tilde(path: &str) -> PathBuf {
    if path == "~" {
        std::env::home_dir().unwrap_or_else(|| PathBuf::from(path))
    } else if let Some(rest) = path.strip_prefix("~/") {
        std::env::home_dir().map_or_else(|| PathBuf::from(path), |home| home.join(rest))
    } else {
        PathBuf::from(path)
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

fn print_completion(shell: &mut Shell, arg: &[String]) -> Outcome {
    let stdout = if arg.is_empty() {
        "no completion specification".to_string()
    } else {
        let cmd = arg.first().unwrap();
        if let Some(result) = shell.completion_script(cmd) {
            format!("complete -C '{result}' {cmd}")
        } else {
            format!("complete: {cmd}: no completion specification")
        }
    };
    Outcome::Output {
        stdout,
        stderr: String::new(),
    }
}

fn register_completion(shell: &mut Shell, args: &[String]) -> Outcome {
    let [first, second, ..] = args else {
        return Outcome::Output {
            stdout: String::new(),
            stderr: String::from("Too many args"),
        };
    };
    shell.completions.insert(second.clone(), first.clone());
    Outcome::Ok
}

fn remove_completion(shell: &mut Shell, args: &[String]) -> Outcome {
    let [first, ..] = args else {
        return Outcome::Output {
            stdout: String::new(),
            stderr: String::from("Too many args"),
        };
    };
    shell.completions.remove(first);
    Outcome::Ok
}
