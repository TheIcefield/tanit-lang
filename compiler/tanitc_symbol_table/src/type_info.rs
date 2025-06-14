use std::collections::BTreeMap;

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
    pub members: BTreeMap<Ident, MemberInfo>,
}
