use tanitc_lexer::token::Token;

use crate::program_ctx::{
    name_ctx::NameCtx, statement_ctx::attributes_ctx::AttributesCtx, type_ctx::TypeCtx,
};

#[derive(Debug, Clone)]
pub struct AliasDefCtx {
    pub attributes_ctx: Box<AttributesCtx>,
    pub alias_tkn: Token, // 'alias'
    pub name_ctx: Box<NameCtx>,
    pub assign_tkn: Token, // =
    pub type_ctx: Box<TypeCtx>,
}
