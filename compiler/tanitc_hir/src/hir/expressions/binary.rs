use std::fmt::Display;

use tanitc_lexer::location::Location;

use crate::hir::expressions::Expression;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperation {
    Add,        // +
    Sub,        // -
    Mul,        // *
    Div,        // /
    Mod,        // %
    Assign,     // =
    BitwiseOr,  // |
    BitwiseXor, // ^
    BitwiseAnd, // &
    ShiftL,     // <<
    ShiftR,     // >>
    LogicalOr,  // ||
    LogicalAnd, // &&
    LogicalEq,  // ==
    LogicalNe,  // !=
    LogicalGt,  // >
    LogicalGe,  // >=
    LogicalLt,  // <
    LogicalLe,  // <=
    ScopeRes,   // ::
    Access,     // .
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr {
    pub location: Location,
    pub operation: BinaryOperation,
    pub lhs: Box<Expression>,
    pub rhs: Box<Expression>,
}

impl Display for BinaryOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                // Arithmetic
                Self::Add => "+",
                Self::Sub => "-",
                Self::Mul => "*",
                Self::Div => "/",
                Self::Mod => "%",
                Self::Assign => "=",

                // Bitwise arithmetic
                Self::BitwiseOr => "|",
                Self::BitwiseXor => "^",
                Self::BitwiseAnd => "&",
                Self::ShiftL => "<<",
                Self::ShiftR => ">>",

                // logical arithmethic
                Self::LogicalOr => "||",
                Self::LogicalAnd => "&&",
                Self::LogicalEq => "==",
                Self::LogicalNe => "!=",
                Self::LogicalGt => ">",
                Self::LogicalGe => ">=",
                Self::LogicalLt => "<",
                Self::LogicalLe => "<=",

                // Special
                Self::ScopeRes => "::",
                Self::Access => ".",
            }
        )
    }
}

impl BinaryOperation {
    pub fn does_mutate(&self) -> bool {
        *self == Self::Assign
    }
}
