use tanitc_lexer::token::Token;

use crate::program_ctx::statement_ctx::{block_ctx::BlockCtx, expression_ctx::ExpressionCtx};

#[derive(Debug, Clone)]
pub struct IfCtx {
    pub if_tkn: Token, // 'if'
    pub expression_ctx: Box<ExpressionCtx>,
    pub block_ctx: Box<BlockCtx>,
}
