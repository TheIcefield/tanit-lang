use tanitc_lexer::token::Token;

#[derive(Debug, Clone)]
pub struct ContinueCtx {
    pub continue_tkn: Token, // 'continue'
}
