use tanitc_attributes::{Mutability, Publicity, Safety};
use tanitc_ident::Name;
use tanitc_lexer::location::Location;

use crate::hir::{
    blocks::Block,
    definitions::{variables::VariableDef, Definition},
    types::Type,
    Hir,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FunctionAttributes {
    pub publicity: Publicity,
    pub safety: Safety,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionParam {
    SelfVal(Mutability),
    SelfRef(Mutability),
    SelfPtr(Mutability),
    Common(VariableDef),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FunctionDef {
    pub location: Location,
    pub attributes: FunctionAttributes,
    pub name: Name,
    pub return_type: Type,
    pub parameters: Vec<FunctionParam>,
    pub body: Option<Box<Block>>,
}

impl From<FunctionDef> for Hir {
    fn from(value: FunctionDef) -> Self {
        Self::Definition(Definition::Func(value))
    }
}
