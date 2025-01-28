use crate::ast::{identifiers::Identifier, types::Type, Ast};
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

impl From<AliasDef> for Ast {
    fn from(value: AliasDef) -> Self {
        Self::AliasDef { node: value }
    }
}

#[cfg(test)]
mod tests;
