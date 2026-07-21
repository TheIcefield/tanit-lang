use tanitc_lexer::token::Token;

use crate::program_ctx::{name_ctx::NameCtx, statement_ctx::attributes_ctx::AttributesCtx};

/// An explicit discriminant assignment on an enum unit (`: value`).
#[derive(Default, Debug, Clone)]
pub struct EnumDefUnitAssignCtx {
    pub colon_tkn: Token, // ':'
    pub value_tkn: Token, // integer
}

/// A single unit (variant) inside an enum definition.
///
/// Optionally carries an explicit integer discriminant via
/// [`EnumDefUnitAssignCtx`].
#[derive(Debug, Clone)]
pub struct EnumDefUnitCtx {
    pub name_ctx: Box<NameCtx>,
    pub assign_ctx: Option<EnumDefUnitAssignCtx>,
}

/// The body of an enum definition — a brace-delimited list of units.
#[derive(Default, Debug, Clone)]
pub struct EnumDefBodyCtx {
    pub lcb_tkn: Token, // '{'
    pub units_ctx: Vec<(
        Option<EnumDefUnitCtx>,
        Option<Token>, // ('\n')?
    )>,
    pub rcb_tkn: Token, // '}'
}

/// An enum definition: `enum Name { Unit1, Unit2: 5, ... }`.
#[derive(Debug, Clone)]
pub struct EnumDefCtx {
    pub attributes_ctx: Box<AttributesCtx>,
    pub enum_tkn: Token, // 'enum'
    pub name_ctx: Box<NameCtx>,
    pub body_ctx: EnumDefBodyCtx,
}
