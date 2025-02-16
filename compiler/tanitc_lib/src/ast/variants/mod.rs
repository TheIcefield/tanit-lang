use crate::ast::{types::Type, Ast};
use tanitc_ident::Ident;
use tanitc_lexer::location::Location;

use std::collections::BTreeMap;

pub mod analyzer;
pub mod codegen;
pub mod parser;
pub mod serializer;

#[derive(Clone, PartialEq, Default)]
pub enum VariantField {
    #[default]
    Common,
    StructLike(BTreeMap<Ident, Type>),
    TupleLike(Vec<Type>),
}

#[derive(Clone, PartialEq, Default)]
pub struct VariantDef {
    pub location: Location,
    pub identifier: Ident,
    pub fields: BTreeMap<Ident, VariantField>,
    pub internals: Vec<Ast>,
}

impl From<VariantDef> for Ast {
    fn from(value: VariantDef) -> Self {
        Self::VariantDef(value)
    }
}

#[cfg(test)]
mod tests;
