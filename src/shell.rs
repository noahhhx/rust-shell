use std::collections::HashMap;

pub struct Shell {
    pub completions: HashMap<String, String>,
    pub last_status: i32,
}

impl Shell {
    #[must_use]
    pub fn new() -> Self {
        Self {
            completions: HashMap::new(),
            last_status: 0,
        }
    }

    pub fn completion_script(&self, cmd: &str) -> Option<&str> {
        self.completions.get(cmd).map(String::as_str)
    }
}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
    }
}
