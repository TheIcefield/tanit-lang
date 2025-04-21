use tanitc_ident::Ident;

pub fn get_variant_data_kind_id(variant_id: Ident) -> Ident {
    Ident::from(format!("__{variant_id}__kind__"))
}

pub fn get_variant_data_type_id(variant_id: Ident) -> Ident {
    Ident::from(format!("__{variant_id}__data__"))
}

pub fn get_variant_data_kind_field_id() -> Ident {
    Ident::from("__kind__".to_string())
}

pub fn get_variant_data_field_id() -> Ident {
    Ident::from("__data__".to_string())
}
