use std::collections::BTreeMap;

use tanitc_attributes::Publicity;
use tanitc_ident::Ident;
use tanitc_lexer::location::Location;
use tanitc_name::NameSpec;

use crate::hir::{definitions::Definition, type_spec::TypeSpec, Hir};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct StructFieldAttributes {
    pub publicity: Publicity,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct StructFieldInfo {
    pub ty: TypeSpec,
    pub attributes: StructFieldAttributes,
}

pub type StructFieldsInfo = BTreeMap<Ident, StructFieldInfo>;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct StructAttributes {
    pub publicity: Publicity,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct StructDef {
    pub location: Location,
    pub attributes: StructAttributes,
    pub name: NameSpec,
    pub fields: StructFieldsInfo,
    pub internals: Vec<Hir>,
}

impl From<StructDef> for Hir {
    fn from(value: StructDef) -> Self {
        Self::Definition(Definition::Struct(value))
    }
}
