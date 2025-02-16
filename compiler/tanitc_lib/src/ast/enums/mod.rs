use crate::ast::Ast;

use tanitc_ident::Ident;
use tanitc_lexer::location::Location;

use std::collections::BTreeMap;

pub mod analyzer;
pub mod codegen;
pub mod parser;
pub mod serializer;

#[derive(Clone, PartialEq, Default)]
pub struct EnumDef {
    pub location: Location,
    pub identifier: Ident,
    pub fields: BTreeMap<Ident, Option<usize>>,
}

impl From<EnumDef> for Ast {
    fn from(value: EnumDef) -> Self {
        Self::EnumDef(value)
    }
}

#[cfg(test)]
mod tests;
