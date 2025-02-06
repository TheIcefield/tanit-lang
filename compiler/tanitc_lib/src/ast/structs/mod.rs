use crate::ast::{types::Type, Ast};
use tanitc_ident::Ident;
use tanitc_lexer::location::Location;

use std::collections::BTreeMap;

pub mod analyzer;
pub mod codegen;
pub mod parser;
pub mod serializer;

#[derive(Default, Clone, PartialEq)]
pub struct StructDef {
    pub location: Location,
    pub identifier: Ident,
    pub fields: BTreeMap<Ident, Type>,
    pub internals: Vec<Ast>,
}

impl From<StructDef> for Ast {
    fn from(value: StructDef) -> Self {
        Self::StructDef(value)
    }
}

#[cfg(test)]
mod tests;
