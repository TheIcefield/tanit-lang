use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperation {
    Ref,    // &
    RefMut, // &mut
    Deref,  // *
    Not,    // !
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
