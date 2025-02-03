use crate::ast::{identifiers::Identifier, types::Type, Ast};

use tanitc_lexer::location::Location;

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

impl From<FunctionDef> for Ast {
    fn from(value: FunctionDef) -> Self {
        Self::FuncDef(value)
    }
}

#[cfg(test)]
mod tests;
