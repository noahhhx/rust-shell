use std::{env, fs, io};
use std::path::Path;
use crate::commands::command::{find_in_path_starts_with, BUILT_IN_COMMANDS};

#[must_use]
pub fn complete(word: &str, command_position: bool) -> Vec<String> {
    let mut candidates = if command_position {
        exe_candidates(word)
    } else {
        file_candidates(word)
    };
    candidates.sort();
    candidates
}

fn exe_candidates(prefix: &str) -> Vec<String> {
    let mut candidates: Vec<String> = BUILT_IN_COMMANDS
        .iter()
        .filter(|c| c.starts_with(prefix))
        .map(ToString::to_string)
        .collect();

    for entry in find_in_path_starts_with(prefix) {
        if !candidates.contains(&entry) {
            candidates.push(entry);
        }
    }
    candidates
}

fn file_candidates(word: &str) -> Vec<String> {
    let cur_dir = env::current_dir().expect("problem reading current directory");

    if word.contains('/') {
        if let Some((path, file_prefix)) = word.rsplit_once('/') {
            let base = Path::new(path);
            let mut files: Vec<String> = list_files_in_dir(base)
                .unwrap_or_default()
                .into_iter()
                .filter(|name| name.starts_with(file_prefix))
                .map(|name| base.join(name).to_string_lossy().into_owned())
                .collect();
            let dirs: Vec<String> = list_dirs_in_dir(base)
                .unwrap_or_default()
                .into_iter()
                .filter(|name| name.starts_with(file_prefix))
                .map(|name| base.join(name + "/").to_string_lossy().into_owned())
                .collect();
            files.extend(dirs.clone());
            return files;
        }
        Vec::new()
    } else {
        let mut files: Vec<String> = list_files_in_dir(&cur_dir)
            .unwrap_or_default()
            .into_iter()
            .filter(|name| name.starts_with(word))
            .collect();
        let dirs: Vec<String> = list_dirs_in_dir(&cur_dir)
            .unwrap_or_default()
            .into_iter()
            .filter(|name| name.starts_with(word))
            .map(|name| name + "/")
            .collect();
        files.extend(dirs.clone());
        files
    }
}

fn list_dirs_in_dir(path: &Path) -> io::Result<Vec<String>> {
    let mut dirs = Vec::new();
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                dirs.push(path.file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_default());
            }
        }
    }
    Ok(dirs)
}

fn list_files_in_dir(path: &Path) -> io::Result<Vec<String>> {
    let mut files = Vec::new();
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                files.push(path.file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_default());
            }
        }
    }
    Ok(files)
}

#[must_use]
pub fn common_prefix(candidates: &[String]) -> String {
    let Some(first) = candidates.first() else {
        return String::new();
    };
    let mut prefix: Vec<char> = first.chars().collect();
    for cand in candidates.iter().skip(1) {
        prefix = prefix
            .into_iter()
            .zip(cand.chars())
            .take_while(|(a, b)| a==b)
            .map(|(a, _)| a)
            .collect();
        if prefix.is_empty() {
            break;
        }
    }
    prefix.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::common_prefix;

    #[test]
    fn lcp_of_nested_prefixes() {
        let cands = ["xyz_foo", "xyz_foo_bar", "xyz_foo_bar_baz"].map(String::from);
        assert_eq!(common_prefix(&cands), "xyz_foo")
    }

    #[test]
    fn lcp_of_equal_length_matches() {
        let cands = ["xyz_foo", "xyz_cat", "xyz_zoo"].map(String::from);
        assert_eq!(common_prefix(&cands), "xyz_")
    }

    #[test]
    fn lcp_of_single_candidate() {
        let cands = [String::from("readme.txt")];
        assert_eq!(common_prefix(&cands), "readme.txt")
    }

    #[test]
    fn lcp_of_nothing() {
        assert_eq!(common_prefix(&[]), "")
    }
}