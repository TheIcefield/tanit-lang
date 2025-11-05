use tanitc_lexer::token::Token;

use crate::program_ctx::statement_ctx::expression_ctx::ExpressionCtx;

#[derive(Debug, Clone)]
pub struct TupleLiteralCtx {
    pub lparen_tkn: Token, // '('
    pub elements: Vec<(
        Option<ExpressionCtx>,
        Option<Token>, // ','?
    )>,
    pub rparen_tkn: Token, // ')'
}
