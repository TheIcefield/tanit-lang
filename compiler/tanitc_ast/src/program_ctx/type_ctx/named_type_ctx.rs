use tanitc_lexer::token::Token;

use crate::program_ctx::{name_ctx::NameCtx, type_ctx::TypeCtx};

#[derive(Debug, Clone)]
pub struct GenericUnitCtx {
    pub type_ctx: Box<TypeCtx>,
    pub comma_tkn: Option<Token>, // (',')?
}

#[derive(Default, Debug, Clone)]
pub struct GenericCtx {
    pub lt_tkn: Token, // '<'
    pub units_ctx: Vec<GenericUnitCtx>,
    pub gt_tkn: Token, // '>'
}

#[derive(Debug, Clone)]
pub struct NamedTypeCtx {
    pub name_ctx: Box<NameCtx>,
    pub generic_ctx: Option<GenericCtx>,
}
