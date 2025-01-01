use crate::ast::{identifiers::Identifier, types::Type};
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

#[cfg(test)]
mod tests;
