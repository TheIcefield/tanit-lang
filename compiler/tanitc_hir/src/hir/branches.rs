use tanitc_lexer::location::Location;

use crate::hir::{blocks::Block, expressions::Expression, Hir};

#[derive(Debug, Clone, PartialEq)]
pub struct Loop {
    pub location: Location,
    pub body: Box<Block>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct While {
    pub location: Location,
    pub body: Box<Block>,
    pub condition: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct If {
    pub location: Location,
    pub body: Box<Block>,
    pub condition: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ElseBody {
    Block(Box<Block>),
    If(Box<If>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Else {
    pub location: Location,
    pub body: ElseBody,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Branch {
    Loop(Loop),
    While(While),
    If(If),
    Else(Else),
}

impl Branch {
    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::Loop(_) => "loop",
            Self::While(_) => "while",
            Self::If(_) => "if",
            Self::Else(_) => "else",
        }
    }

    pub fn location(&self) -> Location {
        match self {
            Self::Loop(branch) => branch.location,
            Self::While(branch) => branch.location,
            Self::If(branch) => branch.location,
            Self::Else(branch) => branch.location,
        }
    }
}

impl From<Branch> for Hir {
    fn from(value: Branch) -> Self {
        Self::BranchStmt(value)
    }
}
