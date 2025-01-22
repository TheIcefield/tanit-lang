use crate::ast::Ast;
use crate::parser::location::Location;
use crate::parser::token::Lexem;

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

#[cfg(test)]
mod tests;
