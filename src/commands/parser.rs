enum State {
    Default,
    InsideSingleQuote,
    InsideDoubleQuote,
    EscapedDefault,
    InsideDoubleQuoteBackslash,
}

pub enum Out {
    StdOut,
    StdErr,
}

const SINGLE_QUOTE: u8 = b'\'';
const DOUBLE_QUOTE: u8 = b'\"';
const BACKSLASH: u8 = b'\\';

pub fn parse(input: &str) -> Vec<String> {
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
                    state = State::Default;
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
        result.push(current);
    }

    result
}

pub enum Redirect {
    File {
        out: Out,
        file: String,
        append: bool,
    },
    None {},
}

pub fn extract_redirects(words: Vec<String>) -> (Vec<String>, Vec<Redirect>) {
    let mut clean: Vec<String> = Vec::new();
    let mut redirects = Vec::new();
    let mut iter = words.iter();

    while let Some(word) = iter.next() {
        match word.as_str() {
            ">" | "1>" => {
                if let Some(file) = iter.next() {
                    redirects.push(Redirect::File {
                        out: Out::StdOut,
                        file: file.to_string(),
                        append: false,
                    });
                }
            }
            _ => clean.push(word.to_string()),
        }
    }
    (clean, redirects)
}
