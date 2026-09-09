use crate::commands::parser::{Out, Redirect, parse};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::exit;

pub struct Statement {
    pub command: Command,
    pub redirects: Vec<Redirect>,
}

impl Statement {
    pub fn command(&self) -> &Command {
        &self.command
    }
}

pub enum Command {
    Exit,
    Echo { args: Vec<String> },
    Type { name: Option<String> },
    Pwd,
    Cd { target: Option<String> },
    External { program: String, args: Vec<String> },
}

pub enum StdReturn {
    StdOut { out_string: String },
    StdErr { err_string: String, exit_code: u8 },
}

impl Command {
    fn from_words(program: &str, args: Vec<String>) -> Command {
        match program {
            "exit" => Command::Exit,
            "echo" => Command::Echo { args },
            "type" => Command::Type {
                name: args.into_iter().next(),
            },
            "pwd" => Command::Pwd,
            "cd" => Command::Cd {
                target: args.into_iter().next(),
            },
            _ => Command::External {
                program: program.to_string(),
                args,
            },
        }
    }

    fn is_builtin(name: &str) -> bool {
        !matches!(Self::from_words(name, Vec::new()),
            Command::External { .. }
        )
    }

    pub fn parse_line(input: &str) -> Option<Statement> {
        let mut words = parse(input);
        let redirects = vec![Redirect {
            std_out_file: String::new(),
            std_err_file: String::new(),
            out: Out::StdOut,
        }];

        if words.is_empty() {
            return None;
        }

        let program = words.remove(0);
        let command = Self::from_words(&program, words);
        Some(Statement { command, redirects })
    }

    pub fn execute(&self) -> Option<StdReturn> {
        match &self {
            Command::Exit => exit(0),
            Command::Echo { args } => echo(args),
            Command::Type { name } => match name {
                None => Some(default_err()),
                Some(n) => type_cmd(n),
            },
            Command::Pwd => pwd(),
            Command::Cd { target } => match target {
                None => Some(default_err()),
                Some(t) => cd(t),
            },
            Command::External { program, args } => external_command(program, args),
        }
    }
}

fn default_err() -> StdReturn {
    StdReturn::StdErr {
        err_string: "Oh no".to_string(),
        exit_code: 1,
    }
}

fn echo(echo_string: &[String]) -> Option<StdReturn> {
    Some(StdReturn::StdOut {
        out_string: echo_string.join(" "),
    })
}

fn type_cmd(type_command: &str) -> Option<StdReturn> {
    let cmd = type_command.trim();
    let ret_val = if Command::is_builtin(&cmd) {
        format!("{type_command} is a shell builtin")
    } else if let Some(path) = find_in_path(cmd) {
        format!("{cmd} is {}", path.display())
    } else {
        format!("{cmd}: not found")
    };
    Some(StdReturn::StdOut {
        out_string: ret_val,
    })
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

fn external_command(cmd: &str, args: &[String]) -> Option<StdReturn> {
    let out_str = if let Some(exe) = find_in_path(cmd) {
        let output = std::process::Command::new(exe.file_name().unwrap())
            .args(args)
            .output()
            .expect("failed to execute process");
        format!(
            "{}",
            String::from_utf8_lossy(output.stdout.trim_ascii_end())
        )
    } else {
        format!("{cmd}: command not found")
    };
    Some(StdReturn::StdOut {
        out_string: out_str,
    })
}

fn pwd() -> Option<StdReturn> {
    let cur_dir = std::env::current_dir().expect("problem reading current directory");
    Some(StdReturn::StdOut {
        out_string: cur_dir.display().to_string(),
    })
}

fn cd(path: &str) -> Option<StdReturn> {
    let target = expand_tilde(path);
    if std::env::set_current_dir(&target).is_err() {
        return Some(StdReturn::StdOut {
            out_string: format!("cd: {}: No such file or directory", target.display()),
        });
    }
    None
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
