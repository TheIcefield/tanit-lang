use tanitc_lexer::token::Token;

use crate::program_ctx::{name_ctx::NameCtx, statement_ctx::expression_ctx::ExpressionCtx};

#[derive(Debug, Clone)]
pub struct NamedCallParamCtx {
    pub name_ctx: Box<NameCtx>,
    pub colon_tkn: Token, // ':'
    pub expression_ctx: Box<ExpressionCtx>,
}

#[derive(Debug, Clone)]
pub struct PositionalCallParamCtx {
    pub expression_ctx: Box<ExpressionCtx>,
}

#[derive(Debug, Clone)]
pub enum CallParamCtx {
    Named(NamedCallParamCtx),
    Positional(PositionalCallParamCtx),
}

#[derive(Debug, Clone)]
pub struct CallParamsCtx {
    pub params: Vec<(
        Option<CallParamCtx>,
        Option<Token>, // ','
    )>,
}

#[derive(Debug, Clone)]
pub struct CallCtx {
    pub expression_ctx: Box<ExpressionCtx>,
    pub lparen_tkn: Token, // '('
    pub params_ctx: CallParamsCtx,
    pub rparen_tkn: Token, // ')'
}
