use tanitc_lexer::token::Token;

use crate::program_ctx::type_ctx::TypeCtx;

#[derive(Debug, Clone)]
pub struct PtrTypeCtx {
    pub star_tkn: Token, // '*'
    pub mut_tkn: Token,  // ('mut' | 'const'),
    pub type_ctx: Box<TypeCtx>,
}
