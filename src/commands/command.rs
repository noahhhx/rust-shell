use crate::commands::parser::{Redirect, extract_redirects, parse};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub struct Statement {
    pub program: String,
    pub args: Vec<String>,
    pub redirects: Vec<Redirect>,
}

impl Statement {
    pub fn parse(input: &str) -> Option<Self> {
        let words = parse(input);
        let (mut words, redirects) = extract_redirects(&words);
        if words.is_empty() {
            return None;
        }
        let program = words.remove(0);
        Some(Self {
            program,
            args: words,
            redirects,
        })
    }
}

pub fn find_in_path_starts_with(prefix: &str) -> Vec<String> {
    let path_var = std::env::var("PATH").unwrap_or_default();
    let mut candidates: Vec<String> = Vec::new();
    for dir in std::env::split_paths(&path_var) {
        for entry in fs::read_dir(&dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|name| name.starts_with(prefix))
                && path.is_file()
                && is_executable(&path)
            {
                candidates.push(
                    path.file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                        .unwrap_or_default(),
                );
            }
        }
    }
    candidates
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
