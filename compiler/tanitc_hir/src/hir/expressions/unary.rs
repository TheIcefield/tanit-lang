use std::fmt::Display;

use tanitc_lexer::{location::Location, token::lexeme::Lexeme};

use crate::hir::expressions::Expression;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperation {
    Add,    // +
    Sub,    // -
    Ref,    // &
    RefMut, // &mut
    Deref,  // *
    Not,    // !
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpr {
    pub location: Location,
    pub operation: UnaryOperation,
    pub node: Box<Expression>,
}

impl TryFrom<Lexeme> for UnaryOperation {
    type Error = String;
    fn try_from(value: Lexeme) -> Result<Self, Self::Error> {
        Ok(match value {
            Lexeme::Ampersand => UnaryOperation::Ref,
            Lexeme::Not => UnaryOperation::Not,
            Lexeme::Star => UnaryOperation::Deref,
            _ => return Err(format!("Unexpected lexeme: {value}")),
        })
    }
}

impl Display for UnaryOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Add => "+",
                Self::Sub => "-",
                Self::Ref => "&",
                Self::RefMut => "&mut",
                Self::Deref => "*",
                Self::Not => "!",
            }
        )
    }
}
