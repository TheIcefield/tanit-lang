use tanitc_lexer::token::Token;

use crate::program_ctx::statement_ctx::{attributes_ctx::AttributesCtx, block_ctx::BlockCtx};

#[derive(Default, Debug, Clone)]
pub struct ExternCtx {
    pub attributes_ctx: Box<AttributesCtx>,
    pub extern_tkn: Token, // 'extern'
    pub abi_tkn: Token,    // '\"' "apiName" '\"'
    pub body_ctx: Box<BlockCtx>,
}
