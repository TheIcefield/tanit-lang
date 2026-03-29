use std::{fmt::Display, iter::Peekable, slice::Iter};

use tanitc_attributes::{Mutability, Safety};
use tanitc_hir::hir::type_spec::{PtrType, RefType, Type};
use tanitc_ident::Ident;
use tanitc_name::{NamePathSegment, NameSpec};

use crate::symbol_table::type_info::TypeMembersInfo;

use super::{
    entry::{Entry, SymbolKind},
    type_info::{MemberInfo, TypeInfo},
};

#[derive(Default, Debug, Clone, Copy)]
pub struct ScopeInfo {
    pub safety: Safety,
    pub is_in_func: bool,
    pub is_in_loop: bool,
}

pub type TableEntries = std::collections::BTreeMap<Ident, Entry>;
pub type TableStack = std::collections::LinkedList<Table>;

#[derive(Default, Debug, Clone)]
pub struct Table {
    table_path: Vec<NamePathSegment>,
    entries: TableEntries,
    stack: TableStack,
    scope_info: ScopeInfo,
}

pub enum LookupError {
    UndefinedInModule {
        namespace: NameSpec,
        id: Ident,
    },
    UndefinedInEnum {
        namespace: NameSpec,
        id: Ident,
    },
    UndefinedInVariant {
        namespace: NameSpec,
        id: Ident,
    },
    RedundantNames {
        namespace: NameSpec,
        tail: NamePathSegment,
    },
    UnexpectedId(NamePathSegment),
    UndefinedId(Ident),
    EmptyNamespec,
}

impl Table {
    pub fn new() -> Self {
        Self {
            table_path: vec![],
            entries: TableEntries::new(),
            stack: TableStack::new(),
            scope_info: ScopeInfo::default(),
        }
    }
}

impl Table {
    pub fn set_path(&mut self, id: Vec<NamePathSegment>) {
        self.table_path = id;
    }

    pub fn get_path(&self) -> Vec<NamePathSegment> {
        self.table_path.clone()
    }

    pub fn get_joined_path(&self, id: Ident) -> Vec<NamePathSegment> {
        let mut new_path = self.get_path();
        new_path.push(id.into());

        new_path
    }

    pub fn enter_scope(&mut self, scope_info: ScopeInfo) {
        let mut table = Table::new();
        table.scope_info = scope_info;

        let current_safety = self.get_safety();

        if scope_info.safety == Safety::Inherited {
            table.scope_info.safety = current_safety;
        }

        self.stack.push_back(table);
    }

    pub fn exit_scope(&mut self) {
        if let Some(mut old_scope) = self.stack.pop_back() {
            for (_entry_name, entry) in old_scope.entries.iter_mut() {
                if entry.is_static {
                    self.insert(std::mem::take(entry));
                }
            }
        }
    }

    pub fn insert(&mut self, entry: Entry) {
        if let Some(back) = self.stack.back_mut() {
            back.entries.insert(entry.id, entry);
        } else {
            self.entries.insert(entry.id, entry);
        }
    }

    pub fn lookup(&self, name: Ident) -> Option<&Entry> {
        let mut res: Option<&Entry> = self.entries.get(&name);

        for scope in self.stack.iter().rev() {
            let entry = scope.entries.get(&name);
            if entry.is_some() {
                res = entry;
            }
        }

        res
    }

    pub fn lookup_mut(&mut self, name: Ident) -> Option<&mut Entry> {
        let mut res: Option<&mut Entry> = self.entries.get_mut(&name);

        for scope in self.stack.iter_mut().rev() {
            let entry = scope.entries.get_mut(&name);
            if entry.is_some() {
                res = entry;
            }
        }

        res
    }

    fn lookup_name_spec_segments(
        &self,
        mut names: Peekable<Iter<NamePathSegment>>,
    ) -> Result<&Entry, LookupError> {
        let next = names.next().cloned().ok_or(LookupError::EmptyNamespec)?;
        let NamePathSegment::Id(next_id) = next else {
            return Err(LookupError::UnexpectedId(next));
        };

        let entry = self
            .lookup(next_id)
            .ok_or(LookupError::UndefinedId(next_id))?;

        if names.peek().is_none() {
            // Return if last
            return Ok(entry);
        };

        // lookup in module
        if let SymbolKind::ModuleDef(data) = &entry.kind {
            return data.table.lookup_name_spec_segments(names).map_err(|err| {
                println!("ERR: {err}");
                match err {
                    LookupError::UndefinedId(id) => LookupError::UndefinedInModule {
                        namespace: data.name.clone(),
                        id,
                    },
                    err => err,
                }
            });
        };

        let next = names.next().cloned().unwrap();
        let NamePathSegment::Id(next_id) = next else {
            return Err(LookupError::UnexpectedId(next.clone()));
        };

        match &entry.kind {
            //lookup in enum definition
            SymbolKind::EnumDef(data) => {
                if let Some(tail) = names.next().cloned() {
                    return Err(LookupError::RedundantNames {
                        namespace: data.name.clone(),
                        tail,
                    });
                }

                data.units
                    .get(&next_id)
                    .ok_or(LookupError::UndefinedInEnum {
                        namespace: data.name.clone(),
                        id: next_id,
                    })
            }

            // lookup in variant definition
            SymbolKind::VariantDef(data) => {
                if let Some(tail) = names.next().cloned() {
                    return Err(LookupError::RedundantNames {
                        namespace: data.name.clone(),
                        tail,
                    });
                }

                data.variants
                    .get(&next_id)
                    .ok_or(LookupError::UndefinedInVariant {
                        namespace: data.name.clone(),
                        id: next_id,
                    })
            }

            // lookup in self
            _ => self
                .lookup(next_id)
                .ok_or(LookupError::UndefinedId(next_id)),
        }
    }

    pub fn lookup_name_spec(&self, name: &NameSpec) -> Result<&Entry, LookupError> {
        self.lookup_name_spec_segments(name.path.iter().peekable())
    }

    pub fn lookup_type(&self, initial_ty: &Type) -> Option<TypeInfo> {
        let mut res: Option<TypeInfo> = None;

        match initial_ty {
            ty if ty.is_common() || ty.is_unit() => {
                return Some(TypeInfo {
                    ty: ty.clone(),
                    mutability: Mutability::default(),
                    members: TypeMembersInfo::new(),
                    is_union: false,
                });
            }
            Type::Array { value_type, size } => {
                let mut internal = self.lookup_type(value_type)?;
                internal.ty = Type::Array {
                    value_type: Box::new(internal.ty),
                    size: *size,
                };
                return Some(internal);
            }
            Type::Ref(ref_type) => {
                let mut internal = self.lookup_type(ref_type.ref_to.as_ref())?;
                internal.ty = Type::Ref(RefType {
                    ref_to: Box::new(internal.ty),
                    mutability: ref_type.mutability,
                });
                return Some(internal);
            }
            Type::Ptr(ptr_type) => {
                let mut internal = self.lookup_type(ptr_type.ptr_to.as_ref())?;
                internal.ty = Type::Ptr(PtrType {
                    ptr_to: Box::new(internal.ty),
                    mutability: ptr_type.mutability,
                });
                return Some(internal);
            }
            _ => {}
        }

        let Some(entry) = self.lookup(Ident::from(initial_ty.to_string())) else {
            for (_, entry) in self.entries.iter() {
                let SymbolKind::ModuleDef(data) = &entry.kind else {
                    continue;
                };

                let Some(info) = data.table.lookup_type(initial_ty) else {
                    continue;
                };

                res = Some(info);
            }

            return res;
        };

        match &entry.kind {
            SymbolKind::StructDef(data) => {
                let mut members = TypeMembersInfo::new();
                for (field_name, field_data) in data.fields.iter() {
                    members.insert(
                        *field_name,
                        MemberInfo {
                            is_public: true,
                            ty: field_data.ty.clone(),
                        },
                    );
                }

                res = Some(TypeInfo {
                    ty: Type::Custom(data.name.clone()),
                    mutability: Mutability::default(),
                    members,
                    is_union: false,
                });
            }
            SymbolKind::UnionDef(data) => {
                let mut members = TypeMembersInfo::new();
                for (field_name, field_data) in data.fields.iter() {
                    members.insert(
                        *field_name,
                        MemberInfo {
                            is_public: true,
                            ty: field_data.ty.clone(),
                        },
                    );
                }

                res = Some(TypeInfo {
                    ty: Type::Custom(data.name.clone()),
                    mutability: Mutability::default(),
                    members,
                    is_union: true,
                });
            }
            SymbolKind::AliasDef(data) => {
                let Some(info) = self.lookup_type(&data.ty) else {
                    return res;
                };

                res = Some(TypeInfo {
                    ty: initial_ty.clone(),
                    mutability: info.mutability,
                    members: info.members,
                    is_union: info.is_union,
                });
            }
            SymbolKind::EnumDef(data) => {
                res = Some(TypeInfo {
                    ty: Type::Custom(data.name.clone()),
                    mutability: Mutability::default(),
                    members: TypeMembersInfo::new(),
                    is_union: false,
                });
            }
            _ => {}
        }

        res
    }

    pub fn get_scope_info(&self) -> ScopeInfo {
        if let Some(back) = self.stack.back() {
            back.scope_info
        } else {
            self.scope_info
        }
    }

    pub fn get_safety(&self) -> Safety {
        self.get_scope_info().safety
    }

    pub fn set_safety(&mut self, safety: Safety) {
        if let Some(back) = self.stack.back_mut() {
            back.scope_info.safety = safety;
        } else {
            self.scope_info.safety = safety
        }
    }
}

impl Display for LookupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyNamespec => write!(f, "empty name specifier"),
            Self::UndefinedId(id) => write!(f, "undefined id: \"{id}\""),
            Self::UnexpectedId(id) => write!(f, "unexpected id: \"{id}\""),
            Self::RedundantNames { namespace, tail } => {
                write!(f, "\"{namespace}\" doesn't contain \"{tail}\"")
            }
            Self::UndefinedInEnum { namespace, id } => {
                write!(f, "enum \"{namespace}\" doesn't contain \"{id}\"")
            }
            Self::UndefinedInVariant { namespace, id } => {
                write!(f, "variant \"{namespace}\" doesn't contain \"{id}\"")
            }
            Self::UndefinedInModule { namespace, id } => {
                write!(f, "module \"{namespace}\" doesn't contain \"{id}\"")
            }
        }
    }
}
