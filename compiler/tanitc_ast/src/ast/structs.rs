use std::collections::BTreeMap;

use tanitc_attributes::Publicity;
use tanitc_ident::Ident;
use tanitc_lexer::location::Location;

use crate::ast::{types::TypeSpec, Ast};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct StructFieldAttributes {
    pub publicity: Publicity,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct StructFieldInfo {
    pub ty: TypeSpec,
    pub attributes: StructFieldAttributes,
}

pub type StructFields = BTreeMap<Ident, StructFieldInfo>;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct StructAttributes {
    pub publicity: Publicity,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct StructDef {
    pub location: Location,
    pub attributes: StructAttributes,
    pub identifier: Ident,
    pub fields: StructFields,
    pub internals: Vec<Ast>,
}

impl From<StructDef> for Ast {
    fn from(value: StructDef) -> Self {
        Self::StructDef(value)
    }
}
