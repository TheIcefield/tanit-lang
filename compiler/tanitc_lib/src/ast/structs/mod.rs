use crate::ast::{identifiers::Identifier, types::Type, Ast};
use crate::parser::location::Location;

use std::collections::BTreeMap;

pub mod analyzer;
pub mod codegen;
pub mod parser;
pub mod serializer;

#[derive(Default, Clone, PartialEq)]
pub struct StructDef {
    pub location: Location,
    pub identifier: Identifier,
    pub fields: BTreeMap<Identifier, Type>,
    pub internals: Vec<Ast>,
}

impl From<StructDef> for Ast {
    fn from(value: StructDef) -> Self {
        Self::StructDef(value)
    }
}

#[cfg(test)]
mod tests;
