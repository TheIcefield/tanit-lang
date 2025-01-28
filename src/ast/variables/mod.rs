use crate::ast::{identifiers::Identifier, types::Type, Ast};
use crate::parser::location::Location;

pub mod analyzer;
pub mod codegen;
pub mod parser;
pub mod serializer;

#[derive(Clone, PartialEq)]
pub struct VariableDef {
    pub location: Location,
    pub identifier: Identifier,
    pub var_type: Type,
    pub is_global: bool,
    pub is_mutable: bool,
}

impl From<VariableDef> for Ast {
    fn from(value: VariableDef) -> Self {
        Self::VariableDef { node: value }
    }
}

#[cfg(test)]
mod tests;
