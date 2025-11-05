use tanitc_lexer::token::Token;

use crate::program_ctx::type_ctx::TypeCtx;

#[derive(Debug, Clone)]
pub struct RefTypeCtx {
    pub ampersand_tkn: Token,   // '&'
    pub mut_tkn: Option<Token>, // ('mut')?,
    pub type_ctx: Box<TypeCtx>,
}
