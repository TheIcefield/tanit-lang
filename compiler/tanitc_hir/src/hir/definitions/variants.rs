use std::collections::BTreeMap;

use tanitc_attributes::Publicity;
use tanitc_ident::{Ident, Name};
use tanitc_lexer::location::Location;

use crate::hir::{
    definitions::{structs::StructFieldsInfo, Definition},
    types::Type,
    Hir,
};

#[derive(Clone, Debug, PartialEq, Default)]
pub enum VariantField {
    #[default]
    Enum,
    Struct(StructFieldsInfo),
    Tuple(Vec<Type>),
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
    pub name: Name,
    pub fields: VariantFields,
    pub internals: Vec<Hir>,
}

impl From<VariantDef> for Hir {
    fn from(value: VariantDef) -> Self {
        Self::Definition(Definition::Variant(value))
    }
}

pub fn get_variant_data_kind_id(variant_id: Name) -> Name {
    Name::from(format!("__{variant_id}__kind__"))
}

pub fn get_variant_data_type_id(variant_id: Name) -> Name {
    Name::from(format!("__{variant_id}__data__"))
}
