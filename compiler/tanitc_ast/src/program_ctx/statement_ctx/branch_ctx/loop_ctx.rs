use tanitc_lexer::token::Token;

use crate::program_ctx::statement_ctx::block_ctx::BlockCtx;

#[derive(Debug, Clone)]
pub struct LoopCtx {
    pub loop_tkn: Token, // 'loop'
    pub block_ctx: Box<BlockCtx>,
}
