use crate::commands::builtins::{BUILTINS, is_executable};
use crate::commands::command::find_in_path_starts_with;
use crate::commands::parser::parse;
use crate::completion::CompletionAction::Show;
use crate::shell::Shell;
use std::path::Path;
use std::process::Command;
use std::{env, fs, io};

pub enum CompletionAction {
    Replace {
        start: usize,
        text: String,
        space: bool,
    },
    Show(Vec<String>),
    None,
}

struct CompletionCtx<'a> {
    word: &'a str,
    word_start: usize,
    command_position: bool,
    command: String,
    first_arg_of: Option<String>,
    previous_word: Option<String>,
}

fn context(line: &str) -> CompletionCtx<'_> {
    let word_start = line.rfind(char::is_whitespace).map_or(0, |i| i + 1);
    let word = &line[word_start..];

    let prefix = &line[..word_start];
    let prev_word_start = prefix
        .trim()
        .rfind(char::is_whitespace)
        .map_or(0, |i| i + 1);
    let previous_word = if prev_word_start != 0 {
        let prev_word_start = &prefix[prev_word_start..];
        Some(prev_word_start.trim().to_string())
    } else {
        None
    };

    let command_position = word_start == 0;
    let parsed = parse(line);
    let command: String = if parsed.is_empty() {
        String::new()
    } else {
        parsed.clone().first().unwrap_or(&String::new()).clone()
    };
    let first_arg_of = if !line.is_empty() && parsed.len() == 1 && line.ends_with(' ') {
        parsed.into_iter().next()
    } else {
        None
    };
    CompletionCtx {
        word,
        word_start,
        command_position,
        command,
        first_arg_of,
        previous_word,
    }
}

#[must_use]
pub fn complete(shell: &Shell, line: &str) -> CompletionAction {
    let ctx = context(line);
    if ctx.command_position {
        reduce(ctx.word_start, ctx.word, &mut exe_candidates(ctx.word))
    } else if let Some(program) = &ctx.first_arg_of
        && let Some(script) = shell.completion_script(program)
    {
        CompletionAction::Replace {
            start: line.len(),
            text: run_script(script, ctx),
            space: true,
        }
    } else if let Some(_previous_word) = &ctx.previous_word
        && let Some(script) = shell.completion_script(&ctx.command)
    {
        CompletionAction::Replace {
            start: line.len() - ctx.word.len(),
            text: run_script(script, ctx),
            space: true,
        }
    } else {
        reduce(ctx.word_start, ctx.word, &mut file_candidates(ctx.word))
    }
}

fn reduce(word_start: usize, word: &str, candidates: &mut [String]) -> CompletionAction {
    let prefix = common_prefix(candidates);

    if candidates.len() == 1 {
        let text: String = candidates[0].clone();
        CompletionAction::Replace {
            start: word_start,
            text: text.clone(),
            space: !text.ends_with(' '),
        }
    } else if prefix.len() > word.len() {
        CompletionAction::Replace {
            start: word_start,
            text: prefix,
            space: false,
        }
    } else if candidates.len() > 1 {
        candidates.sort();
        Show(candidates.to_owned())
    } else {
        CompletionAction::None
    }
}

fn run_script(cmd: &str, ctx: CompletionCtx) -> String {
    let path = Path::new(cmd);
    let first_arg = ctx.command;
    let word = ctx.word.to_string();
    let previosu_word = ctx.previous_word.unwrap_or_default();
    let args = vec![first_arg, word, previosu_word.clone()];

    if path.is_file() && is_executable(path) {
        let output = Command::new(path)
            .args(args)
            .output()
            .expect("uh fucking oh");
        return String::from_utf8_lossy(output.stdout.trim_ascii_end()).to_string();
    }
    String::new()
}

fn exe_candidates(prefix: &str) -> Vec<String> {
    let mut candidates: Vec<String> = BUILTINS
        .iter()
        .filter(|c| c.name.starts_with(prefix))
        .map(|t| t.name.to_string())
        .collect();

    for entry in find_in_path_starts_with(prefix) {
        if !candidates.contains(&entry) {
            candidates.push(entry);
        }
    }
    candidates
}

fn file_candidates(word: &str) -> Vec<String> {
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
        let cur_dir = env::current_dir().expect("problem reading current directory");
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
                dirs.push(
                    path.file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                        .unwrap_or_default(),
                );
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
                files.push(
                    path.file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                        .unwrap_or_default(),
                );
            }
        }
    }
    Ok(files)
}

fn common_prefix(candidates: &[String]) -> String {
    let Some(first) = candidates.first() else {
        return String::new();
    };
    let mut prefix: Vec<char> = first.chars().collect();
    for cand in candidates.iter().skip(1) {
        prefix = prefix
            .into_iter()
            .zip(cand.chars())
            .take_while(|(a, b)| a == b)
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
    use crate::{completion::complete, shell::Shell};

    use super::common_prefix;

    #[test]
    fn test_test() {
        let mut shell = Shell::default();
        let line = "git remote set";
        shell
            .completions
            .insert("git".to_string(), "test".to_string());

        complete(&shell, line);
    }

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
