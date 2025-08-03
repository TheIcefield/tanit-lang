use tanitc_lexer::location::Location;

use crate::ast::Ast;

#[derive(Debug, Clone, PartialEq)]
pub enum ControlFlowKind {
    Return { ret: Option<Box<Ast>> },
    Break { ret: Option<Box<Ast>> },
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

impl From<ControlFlow> for Ast {
    fn from(value: ControlFlow) -> Self {
        Self::ControlFlow(value)
    }
}
