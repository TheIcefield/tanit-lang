use tanitc_lexer::token::Token;

use crate::program_ctx::{
    name_ctx::NameCtx,
    statement_ctx::{attributes_ctx::AttributesCtx, block_ctx::BlockCtx},
};

/// The body of an `impl` block — a brace-delimited block containing
/// method definitions.
#[derive(Default, Debug, Clone)]
pub struct ImplDefBodyCtx {
    pub block_ctx: Box<BlockCtx>,
}

/// An `impl` block: `impl TypeName { methods... }`.
///
/// Associates methods with a named type.
#[derive(Debug, Clone)]
pub struct ImplDefCtx {
    pub attributes_ctx: Box<AttributesCtx>,
    pub impl_tkn: Token, // 'impl'
    pub name_ctx: Box<NameCtx>,
    pub body_ctx: ImplDefBodyCtx,
}
