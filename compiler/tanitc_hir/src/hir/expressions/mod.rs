use tanitc_lexer::location::Location;

use crate::hir::{
    expressions::{
        binary::BinaryExpr, call::CallExpr, conversion::ConversionExpr, indexing::IndexingExpr,
        literal::Literal, member_access::MemberAccessExpr, unary::UnaryExpr, variable::Variable,
    },
    Hir,
};

pub mod binary;
pub mod call;
pub mod conversion;
pub mod indexing;
pub mod literal;
pub mod member_access;
pub mod unary;
pub mod variable;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Unary(UnaryExpr),
    Binary(BinaryExpr),
    MemberAccess(MemberAccessExpr),
    Conversion(ConversionExpr),
    Indexing(IndexingExpr),
    Call(CallExpr),
    Literal(Literal),
    Variable(Variable),
}

impl Expression {
    pub fn location(&self) -> Location {
        match self {
            Self::Unary(expr) => expr.location,
            Self::Binary(expr) => expr.location,
            Self::MemberAccess(expr) => expr.location,
            Self::Conversion(expr) => expr.location,
            Self::Indexing(expr) => expr.location,
            Self::Call(call) => call.location,
            Self::Variable(var) => var.location,
            Self::Literal(lit) => lit.location(),
        }
    }

    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::Unary(_) => "unary-expression",
            Self::Binary(_) => "binary-expression",
            Self::MemberAccess(_) => "member-access-expression",
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
