pub enum Outcome {
    Quit(i32),
    Output { stdout: String, stderr: String },
    Ok,
}

impl Outcome {
    pub fn status(&self) -> i32 {
        match self {
            Self::Quit(n) => *n,
            _ => 0,
        }
    }

    pub fn from_err(stderr: String) -> Self {
        Outcome::Output {
            stdout: String::new(),
            stderr,
        }
    }
}
