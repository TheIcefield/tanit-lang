#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Location {
    pub row: usize,
    pub col: usize,
}

impl Location {
    pub fn new() -> Self {
        Self { row: 1, col: 1 }
    }

    pub fn new_line(&mut self) {
        self.row += 1;
        self.col = 1;
    }

    pub fn shift(&mut self) {
        self.col += 1;
    }
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.row, self.col)
    }
}

impl Default for Location {
    fn default() -> Self {
        Self::new()
    }
}
