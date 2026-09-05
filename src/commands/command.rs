use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::{fs, io};

const BUILT_IN_COMMANDS: [&str; 3] = ["exit", "echo", "type"];

pub enum Command {
    Exit,
    Echo { echo_string: String },
    Type { command_name: String },
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
