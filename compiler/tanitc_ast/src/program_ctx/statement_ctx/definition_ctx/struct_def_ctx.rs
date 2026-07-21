use tanitc_lexer::token::Token;

use crate::program_ctx::{
    name_ctx::NameCtx, statement_ctx::attributes_ctx::AttributesCtx, type_ctx::TypeCtx,
};

/// A single field in a struct definition: `[pub] name: Type`.
#[derive(Debug, Clone)]
pub struct StructDefFieldCtx {
    pub pub_tkn: Option<Token>, // ('pub')?
    pub name_ctx: Box<NameCtx>,
    pub colon_tkn: Token, // ':'
    pub type_ctx: Box<TypeCtx>,
}

/// The body of a struct definition — a brace-delimited list of fields.
#[derive(Default, Debug, Clone)]
pub struct StructDefBodyCtx {
    pub lcb_tkn: Token, // '{'
    pub fields_ctx: Vec<(
        Option<StructDefFieldCtx>,
        Option<Token>, // ('\n')?
    )>,
    pub rcb_tkn: Token, // '}'
}

/// A struct definition: `struct Name { fields... }`.
#[derive(Debug, Clone)]
pub struct StructDefCtx {
    pub attributes_ctx: Box<AttributesCtx>,
    pub struct_tkn: Token, // 'struct'
    pub name_ctx: Box<NameCtx>,
    pub body_ctx: StructDefBodyCtx,
}
