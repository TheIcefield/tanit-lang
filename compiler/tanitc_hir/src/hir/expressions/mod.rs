use tanitc_lexer::location::Location;

use crate::hir::{
    expressions::{
        binary::BinaryExpr, call::CallExpr, conversion::ConversionExpr, indexing::IndexingExpr,
        literal::Literal, unary::UnaryExpr, variable::Variable,
    },
    Hir,
};

pub mod binary;
pub mod call;
pub mod conversion;
pub mod indexing;
pub mod literal;
pub mod unary;
pub mod variable;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Unary(UnaryExpr),
    Binary(BinaryExpr),
    Conversion(ConversionExpr),
    Indexing(IndexingExpr),
    Call(CallExpr),
    Literal(Literal),
    Variable(Variable),
}

impl Expression {
    pub fn location(&self) -> Location {
        match self {
            Self::Binary(expr) => expr.location,
            Self::Unary(expr) => expr.location,
            Self::Conversion(expr) => expr.location,
            Self::Indexing(expr) => expr.location,
            Self::Call(call) => call.location,
            Self::Variable(var) => var.location,
            Self::Literal(lit) => lit.location(),
        }
    }

    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::Binary(_) => "binary-expression",
            Self::Unary(_) => "unary-expression",
            Self::Conversion(_) => "conversion",
            Self::Indexing(_) => "indexing",
            Self::Call(_) => "call",
            Self::Variable(_) => "variable",
            Self::Literal(lit) => lit.kind_str(),
        }
    }
}

impl From<Expression> for Hir {
    fn from(value: Expression) -> Self {
        Self::Expression(value)
    }
}
