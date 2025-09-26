use std::path::{Path, PathBuf};

const DEFAULT_ROW: usize = 1;
const DEFAULT_COL: usize = 1;

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Location {
    pub path: PathBuf,
    pub row: usize,
    pub col: usize,
}

impl Location {
    pub fn new(path: &Path) -> Self {
        Self {
            path: PathBuf::from(path),
            row: DEFAULT_ROW,
            col: DEFAULT_COL,
        }
    }

    pub fn new_line(&mut self) {
        self.row += 1;
        self.col = DEFAULT_COL;
    }

    pub fn shift(&mut self) {
        self.col += 1;
    }
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.path.display(), self.row, self.col)
    }
}

impl Default for Location {
    fn default() -> Self {
        Self {
            path: PathBuf::from(""),
            row: DEFAULT_ROW,
            col: DEFAULT_COL,
        }
    }
}
