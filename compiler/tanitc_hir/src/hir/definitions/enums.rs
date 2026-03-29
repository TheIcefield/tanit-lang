use std::collections::BTreeMap;

use tanitc_attributes::Publicity;
use tanitc_ident::Ident;
use tanitc_lexer::location::Location;
use tanitc_name::NameSpec;

use crate::hir::{definitions::Definition, Hir};

pub type EnumUnitValue = Option<usize>;
pub type EnumUnits = BTreeMap<Ident, EnumUnitValue>;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct EnumAttributes {
    pub publicity: Publicity,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct EnumDef {
    pub location: Location,
    pub attributes: EnumAttributes,
    pub name: NameSpec,
    pub units: EnumUnits,
}

impl From<EnumDef> for Hir {
    fn from(value: EnumDef) -> Self {
        Self::Definition(Definition::Enum(value))
    }
}
