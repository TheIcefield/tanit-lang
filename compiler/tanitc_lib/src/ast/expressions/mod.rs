use crate::ast::Ast;

use tanitc_lexer::{location::Location, token::Lexem};

pub mod analyzer;
pub mod codegen;
pub mod parser;
pub mod serializer;

#[derive(Clone, PartialEq)]
pub enum ExpressionType {
    Unary {
        operation: Lexem,
        node: Box<Ast>,
    },
    Binary {
        operation: Lexem,
        lhs: Box<Ast>,
        rhs: Box<Ast>,
    },
}

#[derive(Clone, PartialEq)]
pub struct Expression {
    pub location: Location,
    pub expr: ExpressionType,
}

impl From<Expression> for Ast {
    fn from(value: Expression) -> Self {
        Self::Expression(value)
    }
}

#[cfg(test)]
mod tests;
