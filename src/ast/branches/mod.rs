use crate::ast::Ast;
use crate::parser::location::Location;

pub mod analyzer;
pub mod codegen;
pub mod parser;
pub mod serializer;

#[derive(Clone, PartialEq)]
pub enum BranchType {
    Loop {
        body: Box<Ast>,
        condition: Option<Box<Ast>>,
    },
    IfElse {
        condition: Box<Ast>,
        main_body: Box<Ast>,
        else_body: Option<Box<Ast>>,
    },
}

#[derive(Clone, PartialEq)]
pub struct Branch {
    location: Location,
    branch: BranchType,
}

#[derive(Clone, PartialEq)]
pub struct Break {
    pub location: Location,
    pub expr: Option<Box<Ast>>,
}

#[derive(Clone, PartialEq)]
pub struct Continue {
    location: Location,
}

#[derive(Clone, PartialEq)]
pub struct Return {
    pub location: Location,
    pub expr: Option<Box<Ast>>,
}
