#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Buffer {
    pub(crate) lines: Vec<String>,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
        }
    }

    pub fn from_text(text: &str) -> Self {
        let mut lines: Vec<String> = text.split('\n').map(|line| line.to_string()).collect();
        if lines.is_empty() {
            lines.push(String::new());
        }
        Self { lines }
    }

    pub fn as_text(&self) -> String {
        self.lines.join("\n")
    }

    pub fn line(&self, row: usize) -> Option<&str> {
        self.lines.get(row).map(|line| line.as_str())
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub(crate) fn line_len_chars(&self, row: usize) -> usize {
        match self.lines.get(row) {
            Some(line) => line.chars().count(),
            None => 0,
        }
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}
