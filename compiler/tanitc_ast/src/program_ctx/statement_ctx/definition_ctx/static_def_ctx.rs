use tanitc_lexer::token::Token;

use crate::program_ctx::{
    name_ctx::NameCtx,
    statement_ctx::{attributes_ctx::AttributesCtx, expression_ctx::ExpressionCtx},
    type_ctx::TypeCtx,
};

#[derive(Debug, Clone)]
pub struct StaticDefTypeCtx {
    pub colon_tkn: Token, // ':'
    pub type_ctx: Box<TypeCtx>,
}

#[derive(Debug, Clone)]
pub struct StaticDefValueCtx {
    pub equal_tkn: Token, // '='
    pub value_ctx: Box<ExpressionCtx>,
}

#[derive(Debug, Clone)]
pub struct StaticDefCtx {
    pub attributes_ctx: Box<AttributesCtx>,
    pub static_tkn: Token, // 'static'
    pub name_ctx: Box<NameCtx>,
    pub mut_tkn: Option<Token>, // ('mut')?
    pub type_ctx: StaticDefTypeCtx,
    pub value_ctx: StaticDefValueCtx,
}
