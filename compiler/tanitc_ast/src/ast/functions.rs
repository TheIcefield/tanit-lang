use tanitc_attributes::{Mutability, Publicity, Safety};
use tanitc_ident::Ident;
use tanitc_lexer::location::Location;

use crate::ast::{blocks::Block, types::TypeSpec, variables::VariableDef, Ast};

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
    pub identifier: Ident,
    pub return_type: TypeSpec,
    pub parameters: Vec<FunctionParam>,
    pub body: Option<Box<Block>>,
}

impl From<FunctionDef> for Ast {
    fn from(value: FunctionDef) -> Self {
        Self::FuncDef(value)
    }
}
