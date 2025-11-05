use tanitc_lexer::token::Token;

use crate::program_ctx::statement_ctx::expression_ctx::ExpressionCtx;

#[derive(Debug, Clone)]
pub struct ReturnCtx {
    pub return_tkn: Token, // 'return'
    pub return_expression_ctx: Option<Box<ExpressionCtx>>,
}
