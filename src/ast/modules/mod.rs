use crate::ast::{identifiers::Identifier, scopes::Scope, Ast};
use crate::parser::location::Location;

pub mod analyzer;
pub mod codegen;
pub mod parser;
pub mod serializer;

#[derive(Clone, PartialEq, Default)]
pub struct ModuleDef {
    pub location: Location,
    pub identifier: Identifier,
    pub is_external: bool,
    pub body: Option<Scope>,
}

impl From<ModuleDef> for Ast {
    fn from(value: ModuleDef) -> Self {
        Self::ModuleDef { node: value }
    }
}

#[cfg(test)]
mod tests;
