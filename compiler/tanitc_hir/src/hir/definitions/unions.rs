use std::collections::BTreeMap;

use tanitc_attributes::Publicity;
use tanitc_ident::Ident;
use tanitc_lexer::location::Location;
use tanitc_name::NameSpec;

use crate::hir::{definitions::Definition, type_spec::TypeSpec, Hir};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct UnionFieldAttributes {
    pub publicity: Publicity,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct UnionFieldInfo {
    pub ty: TypeSpec,
    pub attributes: UnionFieldAttributes,
}

pub type UnionFieldsInfo = BTreeMap<Ident, UnionFieldInfo>;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct UnionAttributes {
    pub publicity: Publicity,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct UnionDef {
    pub location: Location,
    pub attributes: UnionAttributes,
    pub name: NameSpec,
    pub fields: UnionFieldsInfo,
    pub internals: Vec<Hir>,
}

impl From<UnionDef> for Hir {
    fn from(value: UnionDef) -> Self {
        Self::Definition(Definition::Union(value))
    }
}
