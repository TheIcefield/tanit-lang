use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    pub row: usize,
    pub col: usize,
}

impl Default for Location {
    fn default() -> Self {
        Self { row: 1, col: 1 }
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}:{}]", self.row, self.col)
    }
}
