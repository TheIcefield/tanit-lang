use crate::ast::{scopes::Scope, Ast};

use tanitc_ident::Ident;
use tanitc_lexer::location::Location;

pub mod analyzer;
pub mod codegen;
pub mod parser;
pub mod serializer;

#[derive(Clone, PartialEq, Default)]
pub struct ModuleDef {
    pub location: Location,
    pub identifier: Ident,
    pub is_external: bool,
    pub body: Option<Scope>,
}

impl From<ModuleDef> for Ast {
    fn from(value: ModuleDef) -> Self {
        Self::ModuleDef(value)
    }
}

#[cfg(test)]
mod tests;
