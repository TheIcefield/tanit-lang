use std::collections::BTreeMap;

use tanitc_ident::Ident;
use tanitc_ty::Type;

pub struct MemberInfo {
    pub is_public: bool,
    pub ty: Type,
}

pub struct TypeInfo {
    pub members: BTreeMap<Ident, MemberInfo>,
}
