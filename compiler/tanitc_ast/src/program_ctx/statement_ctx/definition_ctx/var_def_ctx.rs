use tanitc_lexer::token::Token;

use crate::program_ctx::{
    name_ctx::NameCtx,
    statement_ctx::{attributes_ctx::AttributesCtx, expression_ctx::ExpressionCtx},
    type_ctx::TypeCtx,
};

#[derive(Debug, Clone)]
pub struct VarDefTypeCtx {
    pub colon_tkn: Token, // ':'
    pub type_ctx: Box<TypeCtx>,
}

#[derive(Debug, Clone)]
pub struct VarDefValueCtx {
    pub equal_tkn: Token, // '='
    pub value_ctx: Box<ExpressionCtx>,
}

#[derive(Debug, Clone)]
pub struct VarDefCtx {
    pub attributes_ctx: Box<AttributesCtx>,
    pub var_tkn: Token,         // 'var'
    pub mut_tkn: Option<Token>, // ('mut')?
    pub name_ctx: Box<NameCtx>,
    pub type_ctx: Option<VarDefTypeCtx>,
    pub value_ctx: Option<VarDefValueCtx>,
}
