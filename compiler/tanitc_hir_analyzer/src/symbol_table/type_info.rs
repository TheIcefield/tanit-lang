use std::fmt::Display;

use tanitc_attributes::Mutability;
use tanitc_hir::hir::type_spec::Type;
use tanitc_ident::Ident;

#[derive(Debug, Clone)]
pub struct MemberInfo {
    pub is_public: bool,
    pub ty: Type,
}

pub type TypeMembersInfo = std::collections::BTreeMap<Ident, MemberInfo>;

#[derive(Default, Debug, Clone)]
pub struct TypeInfo {
    pub ty: Type,
    pub is_union: bool,
    pub mutability: Mutability,
    pub members: TypeMembersInfo,
}

impl Display for TypeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ty)
    }
}
