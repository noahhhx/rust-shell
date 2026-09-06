use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::{fs, io};

const BUILT_IN_COMMANDS: [&str; 5] = ["exit", "echo", "type", "pwd", "cd"];

pub enum Command {
    Exit,
    Echo { echo_string: String },
    Type { command_name: String },
    Pwd {},
    Cd { arg: String },
    NotFound { input: String, args: Vec<String> },
}

impl Command {
    pub fn from_input(input: &str) -> Self {
        let input = input.trim();
        let (command, args) = match input.split_once(" ") {
            None => (input, ""),
            Some((command, args)) => (command, args),
        };

        match command {
            "echo" => Self::Echo {
                echo_string: args.to_string(),
            },
            "exit" => Self::Exit,
            "type" => Self::Type {
                command_name: args.to_string(),
            },
            "pwd" => Self::Pwd {},
            "cd" => Self::Cd {
                arg: args.to_string(),
            },
            _ => Self::NotFound {
                input: command.to_string(),
                args: args.split(" ").map(|s| s.to_string()).collect(),
            },
        }
    }

    pub fn execute(&self) {
        match &self {
            Command::Exit => exit(0),
            Command::Echo { echo_string } => {
                echo(echo_string);
            }
            Command::Type { command_name } => {
                type_cmd(command_name);
            }
            Command::Pwd {} => {
                pwd();
            }
            Command::Cd { arg } => {
                cd(arg);
            }
            Command::NotFound { input, args } => {
                external_command(input, args);
            }
        }
        io::stdout().flush().unwrap();
    }
}

fn echo(echo_string: &String) {
    println!("{echo_string}")
}

fn type_cmd(type_command: &String) {
    let cmd = type_command.trim();
    if BUILT_IN_COMMANDS.contains(&cmd) {
        println!("{type_command} is a shell builtin")
    } else if let Some(path) = find_in_path(cmd) {
        println!("{cmd} is {}", path.display())
    } else {
        println!("{cmd}: not found")
    }
}

fn find_in_path(cmd: &str) -> Option<PathBuf> {
    let path_var = std::env::var("PATH").unwrap_or_default();
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(cmd);
        if candidate.is_file() && is_executable(&candidate) {
            return Some(candidate);
        }
    }
    None
}

fn is_executable(path: &Path) -> bool {
    match fs::metadata(path) {
        Ok(metadata) => {
            let mode = metadata.permissions().mode();
            mode & 0o111 != 0
        }
        Err(_) => false,
    }
}

fn external_command(cmd: &str, args: &Vec<String>) {
    if let Some(exe) = find_in_path(cmd) {
        let output = std::process::Command::new(exe.file_name().unwrap())
            .args(args)
            .output()
            .expect("failed to execute process");
        println!(
            "{}",
            String::from_utf8_lossy(output.stdout.trim_ascii_end())
        )
    } else {
        println!("{}: command not found", cmd);
    }
}

fn pwd() {
    let cur_dir = std::env::current_dir().expect("problem reading current directory");
    println!("{}", cur_dir.display())
}

fn cd(path: &str) {
    let target = expand_tilde(path);
    if let Err(e) = std::env::set_current_dir(&target) {
        println!("cd: {}: No such file or directory", target.display());
    }
}

fn expand_tilde(path: &str) -> PathBuf {
    if path == "~" {
        std::env::home_dir().unwrap_or_else(|| PathBuf::from(path))
    } else if let Some(rest) = path.strip_prefix("~/") {
        std::env::home_dir()
            .map(|home| home.join(rest))
            .unwrap_or_else(|| PathBuf::from(path))
    } else {
        PathBuf::from(path)
    }
}
