use std::path::{Path, PathBuf};

use tanitc_path::PathId;

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Location {
    pub path: PathId,
    pub row: usize,
    pub col: usize,
}

impl Location {
    pub fn new(path: &Path) -> Self {
        Self {
            path: PathId::from(PathBuf::from(path)),
            row: 0,
            col: 0,
        }
    }

    pub fn new_line(&mut self) {
        self.row += 1;
        self.col = 0;
    }

    pub fn shift(&mut self) {
        self.col += 1;
    }
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.path, self.row + 1, self.col + 1)
    }
}

impl Default for Location {
    fn default() -> Self {
        Self {
            path: PathId::from(PathBuf::from("")),
            row: 0,
            col: 0,
        }
    }
}
