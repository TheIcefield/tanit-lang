use tanitc_lexer::location::Location;

use crate::ast::{functions::FunctionDef, Ast};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ExternDef {
    pub location: Location,
    pub abi_name: String,
    pub functions: Vec<FunctionDef>,
}

impl From<ExternDef> for Ast {
    fn from(value: ExternDef) -> Self {
        Self::ExternDef(value)
    }
}
