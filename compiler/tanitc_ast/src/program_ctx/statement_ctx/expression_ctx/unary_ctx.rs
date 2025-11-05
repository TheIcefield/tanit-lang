use tanitc_lexer::token::Token;

use crate::program_ctx::statement_ctx::expression_ctx::ExpressionCtx;

#[derive(Debug, Clone)]
pub enum UnaryOpCtx {
    Add(Token),                // '+'
    Sub(Token),                // '-'
    Ref(Token, Option<Token>), // '&' 'mut'?
}

#[derive(Debug, Clone)]
pub struct UnaryCtx {
    pub unary_op_ctx: UnaryOpCtx,
    pub expression_ctx: Box<ExpressionCtx>,
}
