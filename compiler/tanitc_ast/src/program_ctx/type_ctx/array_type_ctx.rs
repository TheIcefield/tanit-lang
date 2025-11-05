use tanitc_lexer::token::Token;

use crate::program_ctx::{statement_ctx::expression_ctx::ExpressionCtx, type_ctx::TypeCtx};

#[derive(Debug, Clone)]
pub struct ArrayTypeLengthCtx {
    pub colon_tkn: Token, // ':'
    pub expression_ctx: Box<ExpressionCtx>,
}

#[derive(Debug, Clone)]
pub struct ArrayTypeCtx {
    pub lsb_tkn: Token,                         // '['
    pub type_ctx: Box<TypeCtx>,                 // some_type
    pub length_ctx: Option<ArrayTypeLengthCtx>, // (':' expr )?
    pub rsb_tkn: Token,                         // ']'
}
