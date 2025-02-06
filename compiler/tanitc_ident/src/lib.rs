use lazy_static::lazy_static;
use std::{
    fmt::{Debug, Display},
    sync::Mutex,
};

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ident(usize);

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
    pub fn is_built_in(&self) -> bool {
        String::from(*self).starts_with("__tanit_compiler__")
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

#[test]
fn ident_test() {
    let first = Ident::from("foo".to_string());
    let second = Ident::from("bar".to_string());
    let third = Ident::from("baz".to_string());

    assert_eq!(first, Ident(0));
    assert_eq!(second, Ident(1));
    assert_eq!(third, Ident(2));
    assert_eq!(third, Ident::from("baz".to_string()));

    assert_eq!(String::from(first), "foo");
    assert_eq!(String::from(second), "bar");
    assert_eq!(String::from(third), "baz");
    assert_eq!(third.to_string(), "baz");
}
