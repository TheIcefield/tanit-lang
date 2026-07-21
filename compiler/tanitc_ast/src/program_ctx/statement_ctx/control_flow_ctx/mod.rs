//! Control-flow statements: `return`, `break`, `continue`.

pub mod break_ctx;
pub mod continue_ctx;
pub mod return_ctx;

/// A control-flow statement that alters execution within a loop or function.
#[derive(Debug, Clone)]
pub enum ControlFlowCtx {
    Return(return_ctx::ReturnCtx),
    Break(break_ctx::BreakCtx),
    Continue(continue_ctx::ContinueCtx),
}

impl ControlFlowCtx {
    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::Return(_) => "return-ctx",
            Self::Break(_) => "break-ctx",
            Self::Continue(_) => "continue-ctx",
        }
    }
}
