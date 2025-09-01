use std::collections::BTreeMap;

use tanitc_attributes::Publicity;
use tanitc_ident::{Ident, Name};
use tanitc_lexer::location::Location;

use crate::ast::Ast;

pub type EnumUnits = BTreeMap<Ident, Option<usize>>;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct EnumAttributes {
    pub publicity: Publicity,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct EnumDef {
    pub location: Location,
    pub attributes: EnumAttributes,
    pub name: Name,
    pub fields: EnumUnits,
}

impl From<EnumDef> for Ast {
    fn from(value: EnumDef) -> Self {
        Self::EnumDef(value)
    }
}
