use std::collections::BTreeMap;

use tanitc_attributes::Publicity;
use tanitc_ident::Ident;
use tanitc_lexer::location::Location;
use tanitc_name::NameSpec;

use crate::hir::{
    definitions::{structs::StructFieldsInfo, Definition},
    type_spec::Type,
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
    pub name: NameSpec,
    pub fields: VariantFields,
    pub internals: Vec<Hir>,
}

impl From<VariantDef> for Hir {
    fn from(value: VariantDef) -> Self {
        Self::Definition(Definition::Variant(value))
    }
}

impl VariantDef {
    pub fn get_variant_data_kind_name(variant_name: &NameSpec) -> NameSpec {
        let mut name = variant_name.clone();
        name.path.push(Ident::from("kind".to_string()).into());

        name
    }

    pub fn get_variant_data_type_name(variant_name: &NameSpec) -> NameSpec {
        let mut name = variant_name.clone();
        name.path.push(Ident::from("data".to_string()).into());

        name
    }

    pub fn get_variant_kind_name(variant_name: &NameSpec, variant_unit_id: Ident) -> NameSpec {
        let mut name = variant_name.clone();
        name.path.push(Ident::from("kind".to_string()).into());
        name.path.push(variant_unit_id.into());

        name
    }

    pub fn get_variant_data_name(variant_name: &NameSpec, variant_unit_id: Ident) -> NameSpec {
        let mut name = variant_name.clone();
        name.path.push(Ident::from("data".to_string()).into());
        name.path.push(variant_unit_id.into());

        name
    }
}
