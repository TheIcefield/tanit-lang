use tanitc_lexer::token::Token;

use crate::program_ctx::{
    name_ctx::{NameCtx, NameSpecCtx},
    statement_ctx::expression_ctx::ExpressionCtx,
};

#[derive(Debug, Clone)]
pub struct StructFieldLiteralCtx {
    pub name_ctx: NameCtx,
    pub colon_tkn: Token, // ':'
    pub expression_ctx: Box<ExpressionCtx>,
}

#[derive(Debug, Clone)]
pub struct StructLiteralCtx {
    pub name_ctx: NameSpecCtx,
    pub lcb_tkn: Token, // '{'
    pub elements: Vec<(
        Option<StructFieldLiteralCtx>,
        Option<Token>, // ','?
    )>,
    pub rcb_tkn: Token, // '}'
}
