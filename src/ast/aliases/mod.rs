use crate::ast::{identifiers::Identifier, types::Type};
use crate::parser::location::Location;

pub mod analyzer;
pub mod codegen;
pub mod parser;
pub mod serializer;

#[derive(Clone, PartialEq)]
pub struct AliasDef {
    pub location: Location,
    pub identifier: Identifier,
    pub value: Type,
}

#[cfg(test)]
mod tests;
