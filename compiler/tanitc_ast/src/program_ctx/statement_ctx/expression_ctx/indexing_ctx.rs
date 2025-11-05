use tanitc_lexer::token::Token;

use crate::program_ctx::statement_ctx::expression_ctx::ExpressionCtx;

#[derive(Debug, Clone)]
pub struct IndexCtx {
    pub expression_ctx: Box<ExpressionCtx>,
}

#[derive(Debug, Clone)]
pub struct IndexingCtx {
    pub expression_ctx: Box<ExpressionCtx>,
    pub lsb_tkn: Token, // '['
    pub index_ctx: IndexCtx,
    pub rsb_tkn: Token, // ']'
}
