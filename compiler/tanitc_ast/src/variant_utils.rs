use tanitc_ident::Ident;

pub fn get_variant_data_kind_id(variant_id: Ident) -> Ident {
    Ident::from(format!("__{variant_id}__kind__"))
}

pub fn get_variant_data_type_id(variant_id: Ident) -> Ident {
    Ident::from(format!("__{variant_id}__data__"))
}
