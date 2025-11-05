use tanitc_lexer::token::Token;

use crate::program_ctx::{statement_ctx::expression_ctx::ExpressionCtx, type_ctx::TypeCtx};

#[derive(Debug, Clone)]
pub struct ConversionCtx {
    pub expression_ctx: Box<ExpressionCtx>,
    pub as_tkn: Token, // 'as'
    pub type_ctx: Box<TypeCtx>,
}
