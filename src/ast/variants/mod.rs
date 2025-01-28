use crate::ast::{identifiers::Identifier, types::Type, Ast};
use crate::parser::location::Location;

use std::collections::BTreeMap;

pub mod analyzer;
pub mod codegen;
pub mod parser;
pub mod serializer;

#[derive(Clone, PartialEq, Default)]
pub enum VariantField {
    #[default]
    Common,
    StructLike(BTreeMap<Identifier, Type>),
    TupleLike(Vec<Type>),
}

#[derive(Clone, PartialEq, Default)]
pub struct VariantDef {
    pub location: Location,
    pub identifier: Identifier,
    pub fields: BTreeMap<Identifier, VariantField>,
    pub internals: Vec<Ast>,
}

impl From<VariantDef> for Ast {
    fn from(value: VariantDef) -> Self {
        Self::VariantDef { node: value }
    }
}

#[cfg(test)]
mod tests;
