use tanitc_lexer::token::Token;

#[derive(Debug, Clone)]
pub struct OperatorCtx {
    pub token: Token, // ('+' | '=' | '&' | ...)
}
