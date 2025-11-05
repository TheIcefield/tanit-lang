use tanitc_lexer::token::Token;

use crate::program_ctx::type_ctx::TypeCtx;

#[derive(Debug, Clone)]
pub struct FuncTypeReturnTypeCtx {
    pub colon_tkn: Token, // ':'
    pub type_ctx: Box<TypeCtx>,
}

#[derive(Debug, Clone)]
pub struct FuncTypeParamCtx {
    pub type_ctx: Box<TypeCtx>,
    pub comma_tkn: Option<Token>, // (',')?
}

#[derive(Debug, Clone)]
pub struct FuncTypeParamsCtx {
    pub lparen_tkn: Token, // '('
    pub parameters: Vec<FuncTypeParamCtx>,
    pub rparen_tkn: Token, // ')'
}

#[derive(Debug, Clone)]
pub struct FuncTypeCtx {
    pub func_tkn: Token, // 'func'
    pub params_ctx: FuncTypeParamsCtx,
    pub return_type: Option<FuncTypeReturnTypeCtx>,
}
