//! Statement-level AST nodes.
//!
//! Every top-level or block-level construct in Tanit is a [`StatementCtx`].
//! This module also contains sub-modules for the major statement categories:
//! definitions, expressions, branches, control flow, blocks, and use imports.

use tanitc_lexer::token::Token;

pub mod attributes_ctx;
pub mod block_ctx;
pub mod branch_ctx;
pub mod control_flow_ctx;
pub mod definition_ctx;
pub mod expression_ctx;
pub mod use_ctx;

/// A single statement in a Tanit program.
///
/// Statements are the building blocks of Tanit's block structure. They
/// encompass definitions (structs, functions, variables, etc.), expressions,
/// branching (`if`/`else`), control flow (`return`/`break`/`continue`),
/// nested blocks, and `use` imports.
#[derive(Debug, Clone)]
pub enum StatementCtx {
    ControlFlow(control_flow_ctx::ControlFlowCtx),
    Definition(definition_ctx::DefinitionCtx),
    Branch(branch_ctx::BranchCtx),
    Block(block_ctx::BlockCtx),
    Expression(expression_ctx::ExpressionCtx),
    Use(use_ctx::UseCtx),
}

impl StatementCtx {
    /// Returns a human-readable tag identifying the variant (e.g.
    /// `"func-def-ctx"`, `"if-ctx"`).
    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::Definition(ctx) => ctx.kind_str(),
            Self::ControlFlow(ctx) => ctx.kind_str(),
            Self::Branch(ctx) => ctx.kind_str(),
            Self::Expression(ctx) => ctx.kind_str(),
            Self::Block(_) => "block-ctx",
            Self::Use(_) => "use-ctx",
        }
    }
}

/// An ordered sequence of statements separated by optional newline tokens.
///
/// Statements that failed to parse are stored as `None` so that subsequent
/// statements can still be collected. Each statement is paired with an
/// optional trailing `EndOfLine` token.
#[derive(Default, Debug, Clone)]
pub struct StatementsCtx {
    pub statements: Vec<(
        Option<StatementCtx>,
        Option<Token>, // '\n'?
    )>,
}
