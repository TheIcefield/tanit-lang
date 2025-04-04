use std::fmt::Debug;

use tanitc_ident::Ident;

pub type Counter = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeUnit {
    Block(Counter),
    Loop(Counter),
    Module(Ident),
    Struct(Ident),
    Union(Ident),
    Variant(Ident),
    Enum(Ident),
    Func(Ident),
}

impl ScopeUnit {
    pub fn can_fold_symbols(&self) -> bool {
        matches!(
            self,
            Self::Module(_) | Self::Struct(_) | Self::Union(_) | Self::Variant(_) | Self::Enum(_)
        )
    }

    pub fn get_id(&self) -> Option<Ident> {
        match self {
            Self::Module(id)
            | Self::Struct(id)
            | Self::Union(id)
            | Self::Variant(id)
            | Self::Enum(id)
            | Self::Func(id) => Some(*id),
            Self::Block(_) | Self::Loop(_) => None,
        }
    }
}

#[derive(Default, Clone, PartialEq, Eq)]
pub struct Scope(pub Vec<ScopeUnit>);

impl Scope {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, block: ScopeUnit) {
        self.0.push(block);
    }

    pub fn pop(&mut self) {
        self.0.pop();
    }

    pub fn iter(&self) -> std::slice::Iter<'_, ScopeUnit> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, ScopeUnit> {
        self.0.iter_mut()
    }
}

impl Debug for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for i in self.0.iter() {
            write!(f, "/{:?}", i)?;
        }
        write!(f, "]")
    }
}
