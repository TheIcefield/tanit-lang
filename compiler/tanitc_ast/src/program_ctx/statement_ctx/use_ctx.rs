use tanitc_lexer::token::Token;

use crate::program_ctx::name_ctx::NameSpecCtx;

#[derive(Debug, Clone)]
pub struct UseCtx {
    pub use_tkn: Token, // 'use'
    pub name_spec_ctx: NameSpecCtx,
}
