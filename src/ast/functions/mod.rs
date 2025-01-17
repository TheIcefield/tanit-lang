use crate::ast::{identifiers::Identifier, types::Type, Ast};
use crate::parser::location::Location;

pub mod analyzer;
pub mod codegen;
pub mod parser;
pub mod serializer;

#[derive(Clone, PartialEq, Default)]
pub struct FunctionDef {
    pub location: Location,
    pub identifier: Identifier,
    pub return_type: Type,
    pub parameters: Vec<Ast>,
    pub body: Option<Box<Ast>>,
}

#[cfg(test)]
mod tests;
