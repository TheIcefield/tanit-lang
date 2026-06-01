use tanitc_attributes::Publicity;
use tanitc_ident::Ident;
use tanitc_lexer::location::Location;
use tanitc_name::NameSpec;

use crate::hir::{definitions::Definition, Hir};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct EnumAttributes {
    pub publicity: Publicity,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct EnumUnit {
    pub ident: Ident,
    pub value: usize,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct EnumDef {
    pub location: Location,
    pub attributes: EnumAttributes,
    pub name: NameSpec,
    pub units: Vec<EnumUnit>,
}

impl From<EnumDef> for Hir {
    fn from(value: EnumDef) -> Self {
        Self::Definition(Definition::Enum(value))
    }
}
