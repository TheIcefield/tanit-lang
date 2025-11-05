use tanitc_lexer::token::Token;

use crate::program_ctx::{
    name_ctx::NameCtx, statement_ctx::attributes_ctx::AttributesCtx, type_ctx::TypeCtx,
};

#[derive(Debug, Clone)]
pub struct UnionDefFieldCtx {
    pub pub_tkn: Option<Token>, // ('pub')?
    pub name_ctx: Box<NameCtx>,
    pub colon_tkn: Token, // ':'
    pub type_ctx: Box<TypeCtx>,
}

#[derive(Default, Debug, Clone)]
pub struct UnionDefBodyCtx {
    pub lcb_tkn: Token, // '{'
    pub fields_ctx: Vec<(
        Option<UnionDefFieldCtx>,
        Option<Token>, // ('\n')?
    )>,
    pub rcb_tkn: Token, // '}'
}

#[derive(Debug, Clone)]
pub struct UnionDefCtx {
    pub attributes_ctx: Box<AttributesCtx>,
    pub union_tkn: Token, // 'union'
    pub name_ctx: Box<NameCtx>,
    pub body_ctx: UnionDefBodyCtx,
}
