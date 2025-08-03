use tanitc_lexer::location::Location;

use crate::ast::Ast;

#[derive(Debug, Clone, PartialEq)]
pub enum BranchKind {
    Loop { body: Box<Ast> },
    While { body: Box<Ast>, condition: Box<Ast> },
    If { body: Box<Ast>, condition: Box<Ast> },
    Else { body: Box<Ast> },
}

impl BranchKind {
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Loop { .. } => "loop",
            Self::While { .. } => "while",
            Self::If { .. } => "if",
            Self::Else { .. } => "else",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Branch {
    pub location: Location,
    pub kind: BranchKind,
}

impl From<Branch> for Ast {
    fn from(value: Branch) -> Self {
        Self::BranchStmt(value)
    }
}
