//! String interning for file paths in the Tanit compiler.
//!
//! This crate provides an efficient mechanism for representing file-system
//! paths as cheaply copyable integer indices ([`PathId`]) instead of
//! heap-allocated [`PathBuf`] values. When a path is converted to a
//! `PathId`, it is stored in a global thread-safe interning table. Duplicate
//! paths are deduplicated so that the same path always maps to the same
//! `PathId` value, and `PathId` values can be compared for equality in O(1).
//!
//! The interning table is pre-seeded with a `"TestLocation"` entry at index 0,
//! which serves as the default (placeholder) path.
//!
//! # Key Types
//!
//! - [`PathId`] — an interned file path. It is a `Copy` type, making it
//!   extremely cheap to store, pass, and compare.
//!
//! # Example
//!
//! ```
//! use tanitc_path::PathId;
//! use std::path::PathBuf;
//!
//! let a = PathId::from(PathBuf::from("src/main.tan"));
//! let b = PathId::from(PathBuf::from("src/main.tan"));
//! assert_eq!(a, b); // same path → same PathId
//!
//! assert_eq!(a.to_string(), "src/main.tan");
//! ```

use lazy_static::lazy_static;
use std::{fmt::Display, path::PathBuf, sync::Mutex};

/// An interned file-system path.
///
/// `PathId` is a newtype wrapper around `usize` representing an index into
/// the global path interning table. Because `PathId` is `Copy`, it is copied
/// in O(1) and can be used efficiently as a key in hash maps and trees.
///
/// Paths are interned via [`From<PathBuf>`]: if the path already exists in
/// the table, the existing `PathId` is returned, ensuring deduplication.
///
/// # Example
///
/// ```
/// use tanitc_path::PathId;
/// use std::path::PathBuf;
///
/// let a = PathId::from(PathBuf::from("lib/core.tan"));
/// let b = PathId::from(PathBuf::from("lib/core.tan"));
/// assert_eq!(a, b); // same path → same PathId
/// ```
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PathId(usize);

impl From<PathBuf> for PathId {
    fn from(value: PathBuf) -> Self {
        let mut ids = PATHS.lock().unwrap();

        for (index, id) in ids.iter().enumerate() {
            if value.eq(id) {
                return Self(index);
            }
        }

        ids.push(value);
        PathId(ids.len() - 1)
    }
}

impl From<PathId> for PathBuf {
    fn from(value: PathId) -> Self {
        value.as_path_buf()
    }
}

impl PathId {
    /// Returns the raw index of this path in the interning table.
    ///
    /// This is useful when using `PathId` as a key in external data structures.
    pub fn index(&self) -> usize {
        self.0
    }

    /// Resolves this `PathId` back to its original [`PathBuf`].
    ///
    /// Returns an empty path if the index is out of bounds.
    pub fn as_path_buf(&self) -> PathBuf {
        let ids = PATHS.lock().unwrap();

        if let Some(p) = ids.get(self.index()) {
            p.clone()
        } else {
            "".into()
        }
    }
}

impl Display for PathId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_path_buf().to_string_lossy())
    }
}

lazy_static! {
    static ref PATHS: Mutex<Vec<PathBuf>> = Mutex::new(vec!["TestLocation".into()]);
}

#[test]
fn path_test() {
    let first = PathId::from(PathBuf::from("foo"));
    let second = PathId::from(PathBuf::from("bar"));
    let third = PathId::from(PathBuf::from("baz"));

    assert_eq!(first.index(), 1);
    assert_eq!(second.index(), 2);
    assert_eq!(third.index(), 3);
    assert_eq!(third, PathId::from(PathBuf::from("baz")));

    assert_eq!(first.to_string(), "foo");
    assert_eq!(second.to_string(), "bar");
    assert_eq!(third.to_string(), "baz");
}
