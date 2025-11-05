use tanitc_lexer::token::Token;

use crate::program_ctx::statement_ctx::{block_ctx::BlockCtx, branch_ctx::if_ctx::IfCtx};

#[derive(Debug, Clone)]
pub enum ElseBodyCtx {
    Block(Box<BlockCtx>),
    If(Box<IfCtx>),
}

#[derive(Debug, Clone)]
pub struct ElseCtx {
    pub else_tkn: Token, // 'else'
    pub body_ctx: ElseBodyCtx,
}
