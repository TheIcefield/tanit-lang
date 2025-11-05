use tanitc_lexer::location::Location;

use crate::hir::{
    definitions::{functions::FunctionDef, Definition},
    Hir,
};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ExternDef {
    pub location: Location,
    pub abi_name: String,
    pub functions: Vec<FunctionDef>,
}

impl From<ExternDef> for Hir {
    fn from(value: ExternDef) -> Self {
        Self::Definition(Definition::Extern(value))
    }
}
