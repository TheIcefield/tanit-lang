use crate::ast::{types::TypeSpec, Ast};

use tanitc_ident::Ident;
use tanitc_lexer::location::Location;

pub mod analyzer;
pub mod codegen;
pub mod parser;
pub mod serializer;

#[derive(Clone, PartialEq)]
pub struct VariableDef {
    pub location: Location,
    pub identifier: Ident,
    pub var_type: TypeSpec,
    pub is_global: bool,
    pub is_mutable: bool,
}

impl From<VariableDef> for Ast {
    fn from(value: VariableDef) -> Self {
        Self::VariableDef(value)
    }
}

#[cfg(test)]
mod tests;
