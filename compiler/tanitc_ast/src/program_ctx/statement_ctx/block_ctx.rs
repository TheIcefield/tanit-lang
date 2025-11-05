use tanitc_lexer::token::Token;

use crate::program_ctx::statement_ctx::{attributes_ctx::AttributesCtx, StatementsCtx};

#[derive(Default, Debug, Clone)]
pub struct BlockCtx {
    pub attributes_ctx: Box<AttributesCtx>,
    pub lcb_tkn: Token, // '{'
    pub statements_ctx: StatementsCtx,
    pub rcb_tkn: Token, // '}'
}
