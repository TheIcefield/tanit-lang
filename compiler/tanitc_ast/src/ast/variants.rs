use std::collections::BTreeMap;

use tanitc_attributes::Publicity;
use tanitc_ident::Ident;
use tanitc_lexer::location::Location;

use crate::ast::{structs::StructFields, types::TypeSpec, Ast};

#[derive(Clone, Debug, PartialEq, Default)]
pub enum VariantField {
    #[default]
    Common,
    StructLike(StructFields),
    TupleLike(Vec<TypeSpec>),
}

pub type VariantFields = BTreeMap<Ident, VariantField>;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct VariantAttributes {
    pub publicity: Publicity,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct VariantDef {
    pub location: Location,
    pub attributes: VariantAttributes,
    pub identifier: Ident,
    pub fields: VariantFields,
    pub internals: Vec<Ast>,
}

impl From<VariantDef> for Ast {
    fn from(value: VariantDef) -> Self {
        Self::VariantDef(value)
    }
}

pub fn get_variant_data_kind_id(variant_id: Ident) -> Ident {
    Ident::from(format!("__{variant_id}__kind__"))
}

pub fn get_variant_data_type_id(variant_id: Ident) -> Ident {
    Ident::from(format!("__{variant_id}__data__"))
}
