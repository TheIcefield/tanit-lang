use crate::ast::{identifiers::Identifier, Ast};
use crate::parser::location::Location;

pub mod analyzer;
pub mod codegen;
pub mod parser;
pub mod serializer;

#[derive(Clone, PartialEq)]
pub struct ModuleDef {
    pub location: Location,
    pub identifier: Identifier,
    pub body: Box<Ast>,
}

#[cfg(test)]
mod tests;
