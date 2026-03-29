use std::fmt::Display;

use tanitc_ident::Ident;
use tanitc_lexer::token::Token;

#[derive(Debug, Clone)]
pub struct NameCtx {
    pub name_tkn: Token, // identifier
}

pub type NameSpecSegmentCtx = (
    Token,         // identifier
    Option<Token>, // '::'?
);

#[derive(Debug, Clone)]
pub struct NameSpecCtx {
    pub names: Vec<NameSpecSegmentCtx>,
}

impl Display for NameCtx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name_tkn.identifier())
    }
}

impl NameCtx {
    pub fn identifier(&self) -> Ident {
        self.name_tkn.identifier()
    }
}
