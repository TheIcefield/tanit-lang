use std::fmt::Display;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mutability {
    #[default]
    Immutable,
    Mutable,
}

impl Mutability {
    pub fn is_mutable(&self) -> bool {
        *self == Self::Mutable
    }

    pub fn is_const(&self) -> bool {
        !self.is_mutable()
    }
}

impl Display for Mutability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Safety {
    #[default]
    Inherited,
    Safe,
    Unsafe,
}

impl Display for Safety {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Publicity {
    #[default]
    Private,
    Public,
}

impl Display for Publicity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
