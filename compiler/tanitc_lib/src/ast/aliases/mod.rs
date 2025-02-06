use crate::ast::{types::Type, Ast};

use tanitc_ident::Ident;
use tanitc_lexer::location::Location;

pub mod analyzer;
pub mod codegen;
pub mod parser;
pub mod serializer;

#[derive(Default, Clone, PartialEq)]
pub struct AliasDef {
    pub location: Location,
    pub identifier: Ident,
    pub value: Type,
}

impl From<AliasDef> for Ast {
    fn from(value: AliasDef) -> Self {
        Self::AliasDef(value)
    }
}

#[cfg(test)]
mod tests;
