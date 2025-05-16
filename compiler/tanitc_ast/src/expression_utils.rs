use std::fmt::Display;

use tanitc_lexer::token::Lexem;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperation {
    Ref,   // &
    Deref, // *
    Not,   // !
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
