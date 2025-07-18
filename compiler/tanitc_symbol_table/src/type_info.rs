use std::{collections::BTreeMap, fmt::Display};

use tanitc_attributes::Mutability;
use tanitc_ident::Ident;
use tanitc_ty::Type;

#[derive(Debug, Clone)]
pub struct MemberInfo {
    pub is_public: bool,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub ty: Type,
    pub mutability: Mutability,
    pub members: BTreeMap<Ident, MemberInfo>,
}

impl Display for TypeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ty)
    }
}
