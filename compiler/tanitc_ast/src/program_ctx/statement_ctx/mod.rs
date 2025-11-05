use tanitc_lexer::token::Token;

pub mod attributes_ctx;
pub mod block_ctx;
pub mod branch_ctx;
pub mod control_flow_ctx;
pub mod definition_ctx;
pub mod expression_ctx;
pub mod use_ctx;

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

#[derive(Default, Debug, Clone)]
pub struct StatementsCtx {
    pub statements: Vec<(
        Option<StatementCtx>,
        Option<Token>, // '\n'?
    )>,
}
