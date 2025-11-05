use tanitc_lexer::token::Token;

use crate::program_ctx::{
    name_ctx::NameCtx,
    statement_ctx::{attributes_ctx::AttributesCtx, expression_ctx::ExpressionCtx},
    type_ctx::TypeCtx,
};

#[derive(Debug, Clone)]
pub struct ConstDefTypeCtx {
    pub colon_tkn: Token, // ':'
    pub type_ctx: Box<TypeCtx>,
}

#[derive(Debug, Clone)]
pub struct ConstDefValueCtx {
    pub equal_tkn: Token, // '='
    pub value_ctx: Box<ExpressionCtx>,
}

#[derive(Debug, Clone)]
pub struct ConstDefCtx {
    pub attributes_ctx: Box<AttributesCtx>,
    pub const_tkn: Token, // 'const'
    pub name_ctx: Box<NameCtx>,
    pub type_ctx: ConstDefTypeCtx,
    pub value_ctx: ConstDefValueCtx,
}
