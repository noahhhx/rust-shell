enum State {
    Default,
    InsideSingleQuote,
    InsideDoubleQuote,
    EscapedDefault,
    InsideDoubleQuoteBackslash,
}

pub struct Input {
    command: String,
    args: Vec<String>,
}

impl Input {
    pub fn command(&self) -> &str {
        &self.command
    }

    pub fn args(&self) -> &Vec<String> {
        &self.args
    }
}

const SINGLE_QUOTE: u8 = b'\'';
const DOUBLE_QUOTE: u8 = b'\"';
const BACKSLASH: u8 = b'\\';

pub fn parse(input: &str) -> Input {
    let mut result: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut state = State::Default;

    let input = input.as_bytes();
    for i in input {
        match state {
            State::Default => {
                if *i == SINGLE_QUOTE {
                    state = State::InsideSingleQuote;
                } else if *i == DOUBLE_QUOTE {
                    state = State::InsideDoubleQuote;
                } else if i.is_ascii_whitespace() {
                    if !current.is_empty() {
                        result.push(current);
                    }
                    current = String::new();
                } else if *i == BACKSLASH {
                    state = State::EscapedDefault;
                } else {
                    current.push(*i as char);
                }
            }
            State::InsideSingleQuote => {
                if *i == SINGLE_QUOTE {
                    state = State::Default;
                } else {
                    current.push(*i as char);
                }
            }
            State::InsideDoubleQuote => {
                if *i == DOUBLE_QUOTE {
                    state = State::Default
                } else if *i == BACKSLASH {
                    state = State::InsideDoubleQuoteBackslash;
                } else {
                    current.push(*i as char);
                }
            }
            State::EscapedDefault => {
                current.push(*i as char);
                state = State::Default;
            }
            State::InsideDoubleQuoteBackslash => {
                if *i == DOUBLE_QUOTE || *i == BACKSLASH {
                    current.push(*i as char);
                } else {
                    current.push('\\');
                    current.push(*i as char);
                }
                state = State::InsideDoubleQuote;
            }
        }
    }

    // Handle last word
    if !current.is_empty() {
        result.push(current.clone());
    }

    let command = result.first().unwrap();
    let mut arguments = Vec::new();
    let mut i = 0;
    for r in &result {
        if i == 0 {
            i += 1;
            continue;
        }
        arguments.push(r.to_string());
    }
    Input {
        command: command.clone(),
        args: arguments,
    }
}
