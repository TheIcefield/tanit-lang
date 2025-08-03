use tanitc_attributes::{Mutability, Publicity};
use tanitc_ident::Ident;
use tanitc_lexer::location::Location;

use crate::ast::{types::TypeSpec, Ast};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct VariableAttributes {
    pub publicity: Publicity,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct VariableDef {
    pub location: Location,
    pub attributes: VariableAttributes,
    pub identifier: Ident,
    pub var_type: TypeSpec,
    pub is_global: bool,
    pub mutability: Mutability,
}

impl From<VariableDef> for Ast {
    fn from(value: VariableDef) -> Self {
        Self::VariableDef(value)
    }
}
