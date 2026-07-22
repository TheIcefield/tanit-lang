//! Source-code position tracking.

use std::path::{Path, PathBuf};

use tanitc_path::PathId;

/// A source-code location within a file.
///
/// `Location` tracks the file path (as an interned [`PathId`]), a zero-based
/// line number (`row`), and a zero-based column number (`col`).
/// It is used by [`Token`](crate::token::Token) to record where each lexeme
/// was found in the source.
///
/// When displayed, the location is formatted as `path:row+1:col+1`
/// (i.e. human-readable 1-based line and column).
///
/// # Example
///
/// ```
/// use tanitc_lexer::location::Location;
/// use std::path::PathBuf;
///
/// let loc = Location::new(&PathBuf::from("main.tt"));
/// assert_eq!(loc.to_string(), "main.tt:1:1");
/// ```
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Location {
    /// Interned file path.
    pub path: PathId,
    /// Zero-based line number.
    pub row: usize,
    /// Zero-based column number.
    pub col: usize,
}

impl Location {
    /// Creates a new location pointing to the given file path.
    ///
    /// Row and column are initialized to 0.
    pub fn new(path: &Path) -> Self {
        Self {
            path: PathId::from(PathBuf::from(path)),
            row: 0,
            col: 0,
        }
    }

    /// Advances to the next line, resetting the column to 0.
    pub fn new_line(&mut self) {
        self.row += 1;
        self.col = 0;
    }

    /// Advances the column by one position.
    pub fn shift(&mut self) {
        self.col += 1;
    }
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.path, self.row + 1, self.col + 1)
    }
}
