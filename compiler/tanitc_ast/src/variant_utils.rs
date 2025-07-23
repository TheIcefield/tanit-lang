use tanitc_ident::Ident;

use crate::name::Name;

pub fn get_variant_data_kind_id(variant_name: &Name) -> Ident {
    Ident::from(format!("__{}__kind__", variant_name.full_name()))
}

pub fn get_variant_data_type_id(variant_name: &Name) -> Ident {
    Ident::from(format!("__{}__data__", variant_name.full_name()))
}
