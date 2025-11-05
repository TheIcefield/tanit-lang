use tanitc_lexer::location::Location;

use crate::hir::{expressions::Expression, Hir};

#[derive(Debug, Clone, PartialEq)]
pub enum ControlFlowKind {
    Return { ret: Option<Box<Expression>> },
    Break { ret: Option<Box<Expression>> },
    Continue,
}

impl ControlFlowKind {
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Continue => "continue",
            Self::Break { .. } => "break",
            Self::Return { .. } => "return",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ControlFlow {
    pub location: Location,
    pub kind: ControlFlowKind,
}

impl From<ControlFlow> for Hir {
    fn from(value: ControlFlow) -> Self {
        Self::ControlFlow(value)
    }
}
