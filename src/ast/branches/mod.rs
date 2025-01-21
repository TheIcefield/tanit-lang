use crate::ast::{expressions::Expression, Ast};
use crate::parser::location::Location;

pub mod analyzer;
pub mod codegen;
pub mod parser;
pub mod serializer;

#[derive(Clone, PartialEq)]
pub enum BranchType {
    Loop { body: Box<Ast> },
    While { body: Box<Ast>, condition: Box<Ast> },
    If { body: Box<Ast>, condition: Box<Ast> },
    Else { body: Box<Ast> },
}

#[derive(Clone, PartialEq)]
pub struct Branch {
    location: Location,
    branch: BranchType,
}

#[derive(Clone, PartialEq)]
pub enum InterupterType {
    Return { ret: Option<Expression> },
    Break { ret: Option<Expression> },
    Continue,
}

#[derive(Clone, PartialEq)]
pub struct Interupter {
    pub location: Location,
    pub interupter: InterupterType,
}

impl InterupterType {
    pub fn to_str(&self) -> &'static str {
        match self {
            InterupterType::Continue => "continue",
            InterupterType::Break { .. } => "break",
            InterupterType::Return { .. } => "return",
        }
    }
}
