//! Branching constructs: `if`, `else`, `loop`, `while`.

pub mod else_ctx;
pub mod if_ctx;
pub mod loop_ctx;
pub mod while_ctx;

/// A branching statement.
///
/// Covers the four branching forms in Tanit: `if`, `else`, `loop`, and
/// `while`. The `else` variant supports chaining with `else if`.
#[derive(Debug, Clone)]
pub enum BranchCtx {
    If(if_ctx::IfCtx),
    Else(else_ctx::ElseCtx),
    Loop(loop_ctx::LoopCtx),
    While(while_ctx::WhileCtx),
}

impl BranchCtx {
    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::If(_) => "if-ctx",
            Self::Else(_) => "else-ctx",
            Self::Loop(_) => "loop-ctx",
            Self::While(_) => "while-ctx",
        }
    }
}
