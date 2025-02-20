use std::fmt::Debug;

#[derive(Default, Clone)]
pub struct Scope(pub Vec<String>);

impl Scope {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, block: String) {
        self.0.push(block);
    }

    pub fn pop(&mut self) {
        self.0.pop();
    }

    pub fn iter(&self) -> std::slice::Iter<'_, String> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, String> {
        self.0.iter_mut()
    }
}

impl Debug for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for i in self.0.iter() {
            write!(f, "/{}", i)?;
        }
        write!(f, "]")
    }
}
