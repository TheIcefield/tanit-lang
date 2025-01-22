use crate::ast::{identifiers::Identifier, scopes::Scope};
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

#[cfg(test)]
mod tests;
