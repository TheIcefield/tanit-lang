//! String interning for Tanit compiler identifiers.
//!
//! This crate provides an efficient mechanism for representing source-code
//! identifier strings (variable names, function names, type names, etc.) as
//! cheaply copyable integer indices ([`Ident`]) instead of heap-allocated
//! strings. When a string is converted to an `Ident`, it is stored in a
//! global thread-safe interning table. Duplicate strings are deduplicated so
//! that the same string always maps to the same `Ident` value.
//!
//! # Key Types
//!
//! - [`Ident`] — an interned identifier. It is a `Copy` type, making it
//!   extremely cheap to store, pass, and compare.
//! - [`Name`] — an identifier with an optional prefix, used for qualified
//!   or namespaced identifiers (e.g. `prefix__identifier`).
//!
//! # Example
//!
//! ```
//! use tanitc_ident::{Ident, Name};
//!
//! // Intern a string
//! let id = Ident::from("main".to_string());
//! assert_eq!(id.to_string(), "main");
//!
//! // Interning the same string again yields the same Ident
//! let id2 = Ident::from("main".to_string());
//! assert_eq!(id, id2);
//!
//! // A name with a prefix
//! let name = Name {
//!     id: Ident::from("foo".to_string()),
//!     prefix: Some(Ident::from("module".to_string())),
//! };
//! assert_eq!(name.to_string(), "module__foo");
//! ```

use lazy_static::lazy_static;
use std::{
    fmt::{Debug, Display},
    sync::Mutex,
};

/// An interned identifier.
///
/// `Ident` is a newtype wrapper around `usize` representing an index into
/// the global interning table. Because `Ident` is `Copy`, it is copied in
/// O(1) and can be used efficiently as a key in hash maps and trees.
///
/// Strings are interned via [`From<String>`]: if the string already exists in
/// the table, the existing `Ident` is returned, ensuring deduplication.
///
/// # Example
///
/// ```
/// use tanitc_ident::Ident;
///
/// let a = Ident::from("x".to_string());
/// let b = Ident::from("x".to_string());
/// assert_eq!(a, b); // same string → same Ident
/// ```
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ident(usize);

/// An identifier with an optional prefix.
///
/// `Name` extends [`Ident`] with the ability to store an optional prefix,
/// which is used for qualified (namespaced) identifiers.
/// When displayed, it formats as `{prefix}__{id}` if a prefix is present,
/// or simply `{id}` otherwise.
///
/// # Example
///
/// ```
/// use tanitc_ident::{Ident, Name};
///
/// let name = Name {
///     id: Ident::from("bar".to_string()),
///     prefix: Some(Ident::from("foo".to_string())),
/// };
/// assert_eq!(name.to_string(), "foo__bar");
/// ```
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Name {
    /// The primary identifier.
    pub id: Ident,
    /// Optional prefix identifier.
    pub prefix: Option<Ident>,
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(prefix) = &self.prefix {
            write!(f, "{prefix}__{}", self.id)
        } else {
            write!(f, "{}", self.id)
        }
    }
}

impl From<String> for Name {
    fn from(value: String) -> Self {
        Self {
            id: Ident::from(value),
            ..Default::default()
        }
    }
}

impl From<Ident> for Name {
    fn from(value: Ident) -> Self {
        Self {
            id: value,
            ..Default::default()
        }
    }
}

impl From<String> for Ident {
    fn from(value: String) -> Self {
        let mut ids = IDENTIFIERS.lock().unwrap();

        for (index, id) in ids.iter().enumerate() {
            if value.eq(id) {
                return Self(index);
            }
        }

        ids.push(value);
        Ident(ids.len() - 1)
    }
}

impl From<Ident> for String {
    fn from(value: Ident) -> Self {
        let ids = IDENTIFIERS.lock().unwrap();

        if let Some(s) = ids.get(value.0) {
            s.clone()
        } else {
            "".to_string()
        }
    }
}

impl Ident {
    /// Returns `true` if this is a compiler-generated (built-in) identifier.
    ///
    /// Built-in identifiers are those whose string starts with the prefix
    /// `"__tanit_compiler__"`. They are used internally by the compiler and
    /// must not conflict with user-defined names.
    ///
    /// # Example
    ///
    /// ```
    /// use tanitc_ident::Ident;
    ///
    /// let user_id = Ident::from("foo".to_string());
    /// assert!(!user_id.is_built_in());
    ///
    /// let built_in = Ident::from("__tanit_compiler__temp".to_string());
    /// assert!(built_in.is_built_in());
    /// ```
    pub fn is_built_in(&self) -> bool {
        String::from(*self).starts_with("__tanit_compiler__")
    }

    /// Returns the raw index of this identifier in the interning table.
    ///
    /// This is useful when using `Ident` as a key in external data structures.
    pub fn index(&self) -> usize {
        self.0
    }
}

impl Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(*self))
    }
}

impl Debug for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}

lazy_static! {
    static ref IDENTIFIERS: Mutex<Vec<String>> = Mutex::new(vec![]);
}
