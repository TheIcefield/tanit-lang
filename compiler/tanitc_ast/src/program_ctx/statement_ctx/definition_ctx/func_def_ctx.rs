use tanitc_lexer::token::Token;

use crate::program_ctx::{
    name_ctx::NameCtx,
    statement_ctx::{attributes_ctx::AttributesCtx, block_ctx::BlockCtx},
    type_ctx::{func_type_ctx::FuncTypeReturnTypeCtx, TypeCtx},
};

#[derive(Debug, Clone)]
pub struct FuncDefCommonParamCtx {
    pub mut_tkn: Option<Token>, // ('mut')?
    pub name_ctx: Box<NameCtx>,
    pub colon_tkn: Token, // ':'
    pub type_ctx: Box<TypeCtx>,
}

#[derive(Debug, Clone)]
pub struct FuncDefSelfRefParamCtx {
    pub ampersand_tkn: Token,   // '&'
    pub mut_tkn: Option<Token>, // ('mut')?
    pub self_tkn: Token,        // 'self'
}

#[derive(Debug, Clone)]
pub struct FuncDefSelfValParamCtx {
    pub mut_tkn: Option<Token>, // ('mut')?
    pub self_tkn: Token,        // 'self'
}

#[derive(Debug, Clone)]
pub enum FuncDefParamKindCtx {
    CommonParam(FuncDefCommonParamCtx),
    SelfRef(FuncDefSelfRefParamCtx),
    SelfVal(FuncDefSelfValParamCtx),
}

impl FuncDefParamKindCtx {
    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::CommonParam(_) => "common-param-ctx",
            Self::SelfRef(_) => "self-ref-param-ctx",
            Self::SelfVal(_) => "self-val-param-ctx",
        }
    }
}

#[derive(Debug, Clone)]
pub struct FuncDefParamCtx {
    pub param_ctx: FuncDefParamKindCtx,
    pub comma_tkn: Option<Token>, // (',')?
}

#[derive(Default, Debug, Clone)]
pub struct FuncDefParamsCtx {
    pub lparen_tkn: Token, // '('
    pub params_ctx: Vec<FuncDefParamCtx>,
    pub rparen_tkn: Token, // ')'
}

#[derive(Debug, Clone)]
pub struct FuncDefCtx {
    pub attributes_ctx: Box<AttributesCtx>,
    pub func_tkn: Token, // 'func'
    pub name_ctx: Box<NameCtx>,
    pub params_ctx: FuncDefParamsCtx,
    pub return_type_ctx: Option<FuncTypeReturnTypeCtx>,
    pub body_ctx: Option<Box<BlockCtx>>,
}
