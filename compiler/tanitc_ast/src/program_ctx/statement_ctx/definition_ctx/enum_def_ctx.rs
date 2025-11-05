use tanitc_lexer::token::Token;

use crate::program_ctx::{name_ctx::NameCtx, statement_ctx::attributes_ctx::AttributesCtx};

#[derive(Default, Debug, Clone)]
pub struct EnumDefUnitAssignCtx {
    pub colon_tkn: Token, // ':'
    pub value_tkn: Token, // integer
}

#[derive(Debug, Clone)]
pub struct EnumDefUnitCtx {
    pub name_ctx: Box<NameCtx>,
    pub assign_ctx: Option<EnumDefUnitAssignCtx>,
}

#[derive(Default, Debug, Clone)]
pub struct EnumDefBodyCtx {
    pub lcb_tkn: Token, // '{'
    pub units_ctx: Vec<(
        Option<EnumDefUnitCtx>,
        Option<Token>, // ('\n')?
    )>,
    pub rcb_tkn: Token, // '}'
}

#[derive(Debug, Clone)]
pub struct EnumDefCtx {
    pub attributes_ctx: Box<AttributesCtx>,
    pub enum_tkn: Token, // 'enum'
    pub name_ctx: Box<NameCtx>,
    pub body_ctx: EnumDefBodyCtx,
}
