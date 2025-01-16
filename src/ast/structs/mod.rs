use crate::ast::{identifiers::Identifier, types::Type, Ast};
use crate::parser::location::Location;

use std::collections::BTreeMap;

pub mod analyzer;
pub mod codegen;
pub mod parser;
pub mod serializer;

#[derive(Clone, PartialEq)]
pub struct StructDef {
    pub location: Location,
    pub identifier: Identifier,
    pub fields: BTreeMap<Identifier, Type>,
    pub internals: Vec<Ast>,
}

#[cfg(test)]
mod tests;
