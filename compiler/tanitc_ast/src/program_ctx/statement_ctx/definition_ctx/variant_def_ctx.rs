use tanitc_lexer::token::Token;

use crate::program_ctx::{
    name_ctx::NameCtx,
    statement_ctx::{
        attributes_ctx::AttributesCtx, definition_ctx::struct_def_ctx::StructDefBodyCtx,
    },
    type_ctx::{tuple_type_ctx::TupleTypeCtx, TypeCtx},
};

/// An enum-style variant field (unit variant): `VariantName`.
#[derive(Debug, Clone)]
pub struct VariantDefEnumFieldCtx {
    pub name_ctx: Box<NameCtx>,
}

/// A sub-field inside a struct-style variant field: `name: Type`.
#[derive(Debug, Clone)]
pub struct VariantDefStructSubfieldCtx {
    pub name_ctx: Box<NameCtx>,
    pub colon_tkn: Token, // ':'
    pub type_ctx: Box<TypeCtx>,
}

/// A struct-style variant field: `VariantName { fields... }`.
#[derive(Debug, Clone)]
pub struct VariantDefStructFieldCtx {
    pub name_ctx: Box<NameCtx>,
    pub struct_body_ctx: Box<StructDefBodyCtx>,
}

/// A tuple-style variant field: `VariantName(Type, Type)`.
#[derive(Debug, Clone)]
pub struct VariantDefTupleFieldCtx {
    pub name_ctx: Box<NameCtx>,
    pub tuple_type_ctx: Box<TupleTypeCtx>,
}

/// A single field inside a variant definition.
///
/// Variants support three field styles: enum (unit), struct (named fields),
/// and tuple (positional types).
#[derive(Debug, Clone)]
pub enum VariantDefFieldCtx {
    Enum(VariantDefEnumFieldCtx),
    Struct(VariantDefStructFieldCtx),
    Tuple(VariantDefTupleFieldCtx),
}

/// The body of a variant definition — a brace-delimited list of fields.
#[derive(Default, Debug, Clone)]
pub struct VariantDefBodyCtx {
    pub lcb_tkn: Token, // '{'
    pub fields_ctx: Vec<(
        Option<VariantDefFieldCtx>,
        Option<Token>, // (',')?
    )>,
    pub rcb_tkn: Token, // '}'
}

/// A variant (tagged union) definition: `variant Name { ... }`.
#[derive(Debug, Clone)]
pub struct VariantDefCtx {
    pub attributes_ctx: Box<AttributesCtx>,
    pub variant_tkn: Token, // 'variant'
    pub name_ctx: Box<NameCtx>,
    pub body_ctx: VariantDefBodyCtx,
}
