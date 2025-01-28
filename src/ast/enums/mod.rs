use crate::ast::{identifiers::Identifier, Ast};
use crate::parser::location::Location;

use std::collections::BTreeMap;

pub mod analyzer;
pub mod codegen;
pub mod parser;
pub mod serializer;

#[derive(Clone, PartialEq, Default)]
pub struct EnumDef {
    pub location: Location,
    pub identifier: Identifier,
    pub fields: BTreeMap<Identifier, Option<usize>>,
}

impl From<EnumDef> for Ast {
    fn from(value: EnumDef) -> Self {
        Self::EnumDef { node: value }
    }
}

#[cfg(test)]
mod tests;
