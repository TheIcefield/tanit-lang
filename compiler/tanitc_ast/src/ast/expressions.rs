use std::fmt::Display;

use tanitc_lexer::{location::Location, token::Lexem};
use tanitc_messages::Message;
use tanitc_ty::Type;

use crate::ast::{types::TypeSpec, Ast};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperation {
    Ref,    // &
    RefMut, // &mut
    Deref,  // *
    Not,    // !
}

impl TryFrom<Lexem> for UnaryOperation {
    type Error = String;
    fn try_from(value: Lexem) -> Result<Self, Self::Error> {
        Ok(match value {
            Lexem::Ampersand => UnaryOperation::Ref,
            Lexem::Not => UnaryOperation::Not,
            Lexem::Star => UnaryOperation::Deref,
            _ => return Err(format!("Unexpected lexem: {value}")),
        })
    }
}

impl Display for UnaryOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Ref => "&",
                Self::RefMut => "&mut",
                Self::Deref => "*",
                Self::Not => "!",
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperation {
    // Arithmetic
    Add, // +
    Sub, // -
    Mul, // *
    Div, // /
    Mod, // %

    // SelfArithmetic
    Assign,    // =
    AddAssign, // +=
    SubAssign, // -=
    MulAssign, // *=
    DivAssign, // /=
    ModAssign, // %=

    // Bitwise arithmetic
    BitwiseOr,  // |
    BitwiseXor, // ^
    BitwiseAnd, // &
    ShiftL,     // <<
    ShiftR,     // >>

    // Self Bitwise arithmetic
    BitwiseOrAssign,     // |=
    BitwiseXorAssign,    // ^=
    BitwiseAndAssign,    // &=
    BitwiseShiftLAssign, // <<=
    BitwiseShiftRAssign, // >>=

    // logical arithmethic
    LogicalOr,  // ||
    LogicalAnd, // &&
    LogicalEq,  // ==
    LogicalNe,  // !=
    LogicalGt,  // >
    LogicalGe,  // >=
    LogicalLt,  // <
    LogicalLe,  // <=

    // Special
    Access, // ::
    Get,    // .
}

impl TryFrom<Lexem> for BinaryOperation {
    type Error = String;
    fn try_from(value: Lexem) -> Result<Self, Self::Error> {
        Ok(match value {
            // Arithmetic
            Lexem::Plus => Self::Add,
            Lexem::Minus => Self::Sub,
            Lexem::Star => Self::Mul,
            Lexem::Slash => Self::Div,
            Lexem::Percent => Self::Mod,

            // Self arithmetic
            Lexem::Assign => Self::Assign,
            Lexem::AddAssign => Self::AddAssign,
            Lexem::SubAssign => Self::SubAssign,
            Lexem::MulAssign => Self::MulAssign,
            Lexem::DivAssign => Self::DivAssign,
            Lexem::ModAssign => Self::ModAssign,

            // Bitwise arithmetic
            Lexem::Stick => Self::BitwiseOr,
            Lexem::Xor => Self::BitwiseXor,
            Lexem::Ampersand => Self::BitwiseAnd,
            Lexem::LShift => Self::ShiftL,
            Lexem::RShift => Self::ShiftR,

            // Bitwise self arithmetic
            Lexem::OrAssign => Self::BitwiseOrAssign,
            Lexem::XorAssign => Self::BitwiseXorAssign,
            Lexem::AndAssign => Self::BitwiseAndAssign,
            Lexem::LShiftAssign => Self::BitwiseShiftLAssign,
            Lexem::RShiftAssign => Self::BitwiseShiftRAssign,

            // logical arithmethic
            Lexem::Or => Self::LogicalOr,
            Lexem::And => Self::LogicalAnd,
            Lexem::Eq => Self::LogicalEq,
            Lexem::Neq => Self::LogicalNe,
            Lexem::Gt => Self::LogicalGt,
            Lexem::Gte => Self::LogicalGe,
            Lexem::Lt => Self::LogicalLt,
            Lexem::Lte => Self::LogicalLe,

            // Special
            Lexem::Dcolon => Self::Access,
            Lexem::Dot => Self::Get,

            // Error
            _ => return Err(format!("Unexpected lexem: {value}")),
        })
    }
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

                // SelfArithmetic
                Self::Assign => "=",
                Self::AddAssign => "+=",
                Self::SubAssign => "-=",
                Self::MulAssign => "*=",
                Self::DivAssign => "/=",
                Self::ModAssign => "%=",

                // Bitwise arithmetic
                Self::BitwiseOr => "|",
                Self::BitwiseXor => "^",
                Self::BitwiseAnd => "&",
                Self::ShiftL => "<<",
                Self::ShiftR => ">>",

                // Self Bitwise arithmetic
                Self::BitwiseOrAssign => "|=",
                Self::BitwiseXorAssign => "^=",
                Self::BitwiseAndAssign => "&=",
                Self::BitwiseShiftLAssign => "<<=",
                Self::BitwiseShiftRAssign => ">>=",

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
                Self::Access => "::",
                Self::Get => ".",
            }
        )
    }
}

impl BinaryOperation {
    pub fn does_mutate(&self) -> bool {
        *self == Self::Assign
            || *self == Self::SubAssign
            || *self == Self::AddAssign
            || *self == Self::DivAssign
            || *self == Self::ModAssign
            || *self == Self::MulAssign
            || *self == Self::BitwiseAndAssign
            || *self == Self::BitwiseOrAssign
            || *self == Self::BitwiseXorAssign
            || *self == Self::BitwiseShiftLAssign
            || *self == Self::BitwiseShiftRAssign
    }
}

impl ExpressionKind {
    pub fn new_unary(operator: Lexem, operand: Box<Ast>) -> Result<Self, Message> {
        let operation = match UnaryOperation::try_from(operator) {
            Ok(operation) => operation,
            Err(err) => return Err(Message::new(&operand.location(), &err)),
        };

        Ok(Self::Unary {
            operation,
            node: operand,
        })
    }

    pub fn new_binary(operator: Lexem, lhs: Box<Ast>, rhs: Box<Ast>) -> Result<Self, Message> {
        let operation = match BinaryOperation::try_from(operator) {
            Ok(operation) => operation,
            Err(err) => return Err(Message::new(&lhs.location(), &err)),
        };

        Ok(match operation {
            BinaryOperation::Access => Self::Access { lhs, rhs },
            BinaryOperation::Get => Self::Get { lhs, rhs },
            _ => Self::Binary {
                operation,
                lhs,
                rhs,
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionKind {
    Unary {
        operation: UnaryOperation,
        node: Box<Ast>,
    },
    Binary {
        operation: BinaryOperation,
        lhs: Box<Ast>,
        rhs: Box<Ast>,
    },
    Conversion {
        lhs: Box<Ast>,
        ty: TypeSpec,
    },
    Access {
        lhs: Box<Ast>,
        rhs: Box<Ast>,
    },
    Get {
        lhs: Box<Ast>,
        rhs: Box<Ast>,
    },
    Indexing {
        lhs: Box<Ast>,
        index: Box<Ast>,
    },
    Term {
        node: Box<Ast>,
        ty: Type,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    pub location: Location,
    pub kind: ExpressionKind,
}

impl From<Expression> for Ast {
    fn from(value: Expression) -> Self {
        Self::Expression(value)
    }
}
