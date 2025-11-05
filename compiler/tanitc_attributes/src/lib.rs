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

impl Safety {
    pub fn is_safe(&self) -> bool {
        Safety::Safe == *self
    }

    pub fn is_unsafe(&self) -> bool {
        Safety::Unsafe == *self
    }

    pub fn is_inherited(&self) -> bool {
        Safety::Inherited == *self
    }
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

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    #[default]
    Local,
    Global,
}

impl Visibility {
    pub fn is_global(&self) -> bool {
        *self == Self::Global
    }

    pub fn is_local(&self) -> bool {
        *self == Self::Local
    }
}

impl Display for Visibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
