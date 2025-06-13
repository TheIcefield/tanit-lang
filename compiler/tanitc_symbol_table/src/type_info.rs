use std::collections::BTreeMap;

use tanitc_ident::Ident;

pub struct MemberInfo {
    pub is_public: bool,
}

pub struct TypeInfo {
    pub members: BTreeMap<Ident, MemberInfo>,
}
