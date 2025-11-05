use tanitc_lexer::token::Token;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct NeverTypeCtx {
    pub excm_tkn: Token, // '!'
}
