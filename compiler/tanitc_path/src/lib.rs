use lazy_static::lazy_static;
use std::{fmt::Display, path::PathBuf, sync::Mutex};

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
    pub fn index(&self) -> usize {
        self.0
    }

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
