use tanitc_lexer::token::Token;

use crate::program_ctx::statement_ctx::expression_ctx::ExpressionCtx;

#[derive(Debug, Clone)]
pub struct ParenCtx {
    pub lparen_tkn: Token, // '('
    pub expression_ctx: Box<ExpressionCtx>,
    pub rparen_tkn: Token, // ')'
}
