use tanitc_attributes::{Mutability, Publicity, Visibility};
use tanitc_ident::Ident;
use tanitc_lexer::location::Location;

use crate::hir::{definitions::Definition, expressions::Expression, types::Type, Hir};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct VariableAttributes {
    pub publicity: Publicity,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct VariableDef {
    pub location: Location,
    pub attributes: VariableAttributes,
    pub identifier: Ident,
    pub var_type: Type,
    pub visibility: Visibility,
    pub mutability: Mutability,
    pub value: Option<Box<Expression>>,
}

impl From<VariableDef> for Hir {
    fn from(value: VariableDef) -> Self {
        Self::Definition(Definition::Variable(value))
    }
}
