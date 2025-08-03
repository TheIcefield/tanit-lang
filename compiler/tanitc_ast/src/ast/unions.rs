use std::collections::BTreeMap;

use tanitc_attributes::Publicity;
use tanitc_ident::Ident;
use tanitc_lexer::location::Location;

use crate::ast::{types::TypeSpec, Ast};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct UnionFieldAttributes {
    pub publicity: Publicity,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct UnionFieldInfo {
    pub ty: TypeSpec,
    pub attributes: UnionFieldAttributes,
}

pub type UnionFields = BTreeMap<Ident, UnionFieldInfo>;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct UnionAttributes {
    pub publicity: Publicity,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct UnionDef {
    pub location: Location,
    pub attributes: UnionAttributes,
    pub identifier: Ident,
    pub fields: UnionFields,
    pub internals: Vec<Ast>,
}

impl From<UnionDef> for Ast {
    fn from(value: UnionDef) -> Self {
        Self::UnionDef(value)
    }
}
