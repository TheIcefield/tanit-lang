use tanitc_lexer::token::Token;

#[derive(Debug, Clone)]
pub struct BreakCtx {
    pub break_tkn: Token, // 'break'
}
