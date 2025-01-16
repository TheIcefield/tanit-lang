use crate::ast::{identifiers::Identifier, types::Type, Ast};
use crate::parser::location::Location;

use std::collections::BTreeMap;

pub mod analyzer;
pub mod codegen;
pub mod parser;
pub mod serializer;

#[derive(Clone, PartialEq)]
pub enum VariantField {
    StructLike(BTreeMap<Identifier, Type>),
    TupleLike(Vec<Type>),
    Common,
}

#[derive(Clone, PartialEq)]
pub struct VariantDef {
    pub location: Location,
    pub identifier: Identifier,
    pub fields: BTreeMap<Identifier, VariantField>,
    pub internals: Vec<Ast>,
}

#[cfg(test)]
mod tests;
