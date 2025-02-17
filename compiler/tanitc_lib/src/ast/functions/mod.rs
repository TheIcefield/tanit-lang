use crate::ast::Ast;

use tanitc_ident::Ident;
use tanitc_lexer::location::Location;

use super::types::TypeSpec;

pub mod analyzer;
pub mod codegen;
pub mod parser;
pub mod serializer;

#[derive(Clone, PartialEq, Default)]
pub struct FunctionDef {
    pub location: Location,
    pub identifier: Ident,
    pub return_type: TypeSpec,
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
