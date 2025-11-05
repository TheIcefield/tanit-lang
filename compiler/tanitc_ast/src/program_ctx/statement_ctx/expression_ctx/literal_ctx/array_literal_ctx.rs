use tanitc_lexer::token::Token;

use crate::program_ctx::statement_ctx::expression_ctx::ExpressionCtx;

#[derive(Debug, Clone)]
pub struct ArrayLiteralCtx {
    pub lsb_tkn: Token, // '['
    pub elements: Vec<(
        Option<ExpressionCtx>,
        Option<Token>, // ','?
    )>,
    pub rsb_tkn: Token, // ']'
}
