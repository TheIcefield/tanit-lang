use tanitc_lexer::token::Token;

use crate::program_ctx::type_ctx::TypeCtx;

#[derive(Debug, Clone)]
pub struct TupleTypeUnitCtx {
    pub type_ctx: Box<TypeCtx>,
    pub comma_tkn: Option<Token>, // (',')?
}

#[derive(Debug, Clone)]
pub struct TupleTypeCtx {
    pub lparen_tkn: Token, // '('
    pub units_ctx: Vec<TupleTypeUnitCtx>,
    pub rparen_tkn: Token, // ')'
}
