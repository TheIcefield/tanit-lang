use tanitc_ident::Ident;
use tanitc_lexer::location::Location;

use crate::hir::{
    definitions::{functions::FunctionDef, Definition},
    Hir,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ImplAttributes {}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ImplDef {
    pub location: Location,
    pub attrs: ImplAttributes,
    pub identifier: Ident,
    pub methods: Vec<FunctionDef>,
}

impl From<ImplDef> for Hir {
    fn from(value: ImplDef) -> Self {
        Self::Definition(Definition::Impl(value))
    }
}
