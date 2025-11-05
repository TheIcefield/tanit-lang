use tanitc_ident::Ident;
use tanitc_lexer::token::Token;

#[derive(Debug, Clone)]
pub struct UseCtx {
    pub use_tkn: Token,     // 'use'
    pub idents: Vec<Ident>, // (Ident)+
}
