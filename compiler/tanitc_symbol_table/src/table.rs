use std::collections::{BTreeMap, LinkedList};

use tanitc_attributes::{Mutability, Safety};
use tanitc_ident::Ident;
use tanitc_ty::Type;

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

#[derive(Default, Debug, Clone)]
pub struct Table {
    entries: BTreeMap<Ident, Entry>,
    stack: LinkedList<Table>,
    scope_info: ScopeInfo,
}

impl Table {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            stack: LinkedList::new(),
            scope_info: ScopeInfo::default(),
        }
    }
}

impl Table {
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
            back.entries.insert(entry.name, entry);
        } else {
            self.entries.insert(entry.name, entry);
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

    pub fn lookup_qualified(
        &self,
        mut names: std::iter::Peekable<std::slice::Iter<Ident>>,
    ) -> Option<&Entry> {
        let next = names.next()?;

        let entry = self.lookup(*next)?;

        // If it was not the last
        if names.peek().is_some() {
            match &entry.kind {
                SymbolKind::ModuleDef(data) => {
                    // lookup qualified in module
                    data.table.lookup_qualified(names)
                }
                SymbolKind::EnumDef(data) => {
                    // get entry from enum definition
                    data.enums.get(names.next().unwrap())
                }
                SymbolKind::VariantDef(data) => {
                    // get entry from variant definition
                    data.variants.get(names.next().unwrap())
                }
                _ => {
                    // lookup in self
                    self.lookup(*names.next().unwrap())
                }
            }
        } else {
            Some(entry)
        }
    }

    pub fn lookup_type(&self, ty: &Type) -> Option<TypeInfo> {
        let mut res: Option<TypeInfo> = None;

        if ty.is_common() {
            return Some(TypeInfo {
                ty: ty.clone(),
                mutability: Mutability::default(),
                members: BTreeMap::new(),
                is_union: false,
            });
        }

        let name = Ident::from(ty.to_string());

        if let Some(entry) = self.lookup(name) {
            match &entry.kind {
                SymbolKind::StructDef(data) => {
                    let mut members = BTreeMap::<Ident, MemberInfo>::new();

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
                        ty: Type::Custom(name.to_string()),
                        mutability: Mutability::default(),
                        members,
                        is_union: false,
                    });
                }
                SymbolKind::UnionDef(data) => {
                    let mut members = BTreeMap::<Ident, MemberInfo>::new();

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
                        ty: Type::Custom(name.to_string()),
                        mutability: Mutability::default(),
                        members,
                        is_union: true,
                    });
                }
                _ => {}
            }
        } else {
            for (_, entry) in self.entries.iter() {
                if let SymbolKind::ModuleDef(data) = &entry.kind {
                    if let Some(info) = data.table.lookup_type(ty) {
                        res = Some(info);
                        continue;
                    }
                }
            }
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

#[test]
fn table_test() {
    /* example:
     * Module Main {       # Main: @s
     *     func bar() { }  # bar:  @s/Main
     *     func main() {   # main: @s/Main
     *         let var = 5 # var:  @s/Main/main
     *     }
     * }
     */
    use tanitc_ty::Type;

    use crate::entry::{FuncDefData, ModuleDefData, VarDefData, VarStorageType};

    let main_mod_id = Ident::from("Main".to_string());
    let main_fn_id = Ident::from("main".to_string());
    let bar_id = Ident::from("bar".to_string());
    let var_id = Ident::from("var".to_string());
    let baz_id = Ident::from("baz".to_string());

    let mut table = Table::new();

    table.insert(Entry {
        name: main_mod_id,
        is_static: true,
        kind: SymbolKind::ModuleDef(ModuleDefData {
            table: Box::new(Table::new()),
        }),
    });

    let main_mod = table.lookup_mut(main_mod_id).unwrap();
    let SymbolKind::ModuleDef(ref mut data) = &mut main_mod.kind else {
        unreachable!()
    };

    let table = &mut data.table;

    {
        table.enter_scope(ScopeInfo {
            safety: Safety::Safe,
            ..Default::default()
        }); // enter Main

        table.insert(Entry {
            name: bar_id,
            is_static: false,
            kind: SymbolKind::from(FuncDefData {
                parameters: vec![],
                return_type: Type::unit(),
                is_virtual: false,
                is_inline: false,
                no_return: true,
                safety: Safety::Safe,
            }),
        });

        {
            table.enter_scope(ScopeInfo {
                safety: Safety::Safe,
                is_in_func: true,
                ..Default::default()
            }); // enter bar

            // check if var not visible in bar
            assert!(table.lookup(var_id).is_none());

            // check if baz not defined in bar
            assert!(table.lookup(baz_id).is_none());

            table.exit_scope(); // exit bar
        }

        table.insert(Entry {
            name: main_fn_id,
            is_static: false,
            kind: SymbolKind::from(FuncDefData {
                parameters: vec![],
                return_type: Type::unit(),
                is_virtual: false,
                is_inline: false,
                no_return: true,
                safety: Safety::Safe,
            }),
        });

        {
            table.enter_scope(ScopeInfo {
                safety: Safety::Safe,
                is_in_func: true,
                ..Default::default()
            }); // enter main

            table.insert(Entry {
                name: var_id,
                is_static: false,
                kind: SymbolKind::from(VarDefData {
                    storage: VarStorageType::Auto,
                    var_type: Type::I32,
                    mutability: Mutability::default(),
                    is_initialization: true,
                }),
            });

            // check if var visible in main
            assert!(table.lookup(var_id).is_some());

            table.exit_scope(); // exit main
        }

        // check if main visible in Main
        assert!(table.lookup(main_fn_id).is_some());

        // check if var not visible in Main
        assert!(table.lookup(var_id).is_none());

        // check if baz not visible in Main
        assert!(table.lookup(baz_id).is_none());
    }
}

#[test]
fn qualified_symbol_test() {
    /* example:
     * Module M1 {       # M1
     *     func f1() { } # M1/f1
     *     func f2() { } # M1/f2
     * }
     * Module M2 {       # M2
     *     func f2() { } # M2/f2
     * }
     */

    use crate::entry::{FuncDefData, ModuleDefData};
    use tanitc_ty::Type;

    let m1_id = Ident::from("M1".to_string());
    let m2_id = Ident::from("M2".to_string());
    let f1_id = Ident::from("f1".to_string());
    let f2_id = Ident::from("f2".to_string());

    let mut table = Table::new();

    table.insert(Entry {
        name: m1_id,
        is_static: true,
        kind: SymbolKind::ModuleDef(ModuleDefData {
            table: Box::new(Table::new()),
        }),
    });

    {
        let m1 = table.lookup_mut(m1_id).unwrap();
        let SymbolKind::ModuleDef(ref mut data) = &mut m1.kind else {
            unreachable!()
        };

        data.table.insert(Entry {
            name: f1_id,
            is_static: false,
            kind: SymbolKind::from(FuncDefData {
                parameters: vec![],
                return_type: Type::unit(),
                is_virtual: false,
                is_inline: false,
                no_return: true,
                safety: Safety::Safe,
            }),
        });

        data.table.insert(Entry {
            name: f2_id,
            is_static: false,
            kind: SymbolKind::from(FuncDefData {
                parameters: vec![],
                return_type: Type::unit(),
                is_virtual: false,
                is_inline: false,
                no_return: true,
                safety: Safety::Safe,
            }),
        });
    }

    table.insert(Entry {
        name: m2_id,
        is_static: true,
        kind: SymbolKind::ModuleDef(ModuleDefData {
            table: Box::new(Table::new()),
        }),
    });

    {
        let m2 = table.lookup_mut(m2_id).unwrap();
        let SymbolKind::ModuleDef(ref mut data) = &mut m2.kind else {
            unreachable!()
        };

        data.table.insert(Entry {
            name: f2_id,
            is_static: false,
            kind: SymbolKind::from(FuncDefData {
                parameters: vec![],
                return_type: Type::unit(),
                is_virtual: false,
                is_inline: false,
                no_return: true,
                safety: Safety::Safe,
            }),
        });
    }

    assert!(table
        .lookup_qualified([m1_id, f2_id].iter().peekable())
        .is_some());
    assert!(table
        .lookup_qualified([m2_id, f1_id].iter().peekable())
        .is_none());
}

#[test]
fn lookup_type_test() {
    /* example:
     * Module M1 {       # M1
     *     Module M2 {   # M1/M2
     *         Struct S1 {
     *             f1: i32
     *             f2: f32
     *         }
     *     }
     * }
     */

    use crate::entry::{ModuleDefData, StructDefData, StructFieldData};
    use tanitc_ty::Type;

    let m1_id = Ident::from("M1".to_string());
    let m2_id = Ident::from("M2".to_string());
    let s1_id = Ident::from("S1".to_string());
    let f1_id = Ident::from("f1".to_string());
    let f2_id = Ident::from("f2".to_string());

    let mut table = Table::new();

    table.insert(Entry {
        name: m1_id,
        is_static: true,
        kind: SymbolKind::ModuleDef(ModuleDefData {
            table: Box::new(Table::new()),
        }),
    });

    {
        let m1 = table.lookup_mut(m1_id).unwrap();
        let SymbolKind::ModuleDef(ref mut data) = &mut m1.kind else {
            unreachable!()
        };

        data.table.insert(Entry {
            name: m2_id,
            is_static: true,
            kind: SymbolKind::ModuleDef(ModuleDefData {
                table: Box::new(Table::new()),
            }),
        });

        {
            let m2 = data.table.lookup_mut(m2_id).unwrap();
            let SymbolKind::ModuleDef(ref mut data) = &mut m2.kind else {
                unreachable!()
            };

            data.table.insert(Entry {
                name: s1_id,
                is_static: false,
                kind: SymbolKind::from(StructDefData {
                    fields: {
                        let mut field = BTreeMap::<Ident, StructFieldData>::new();

                        field.insert(f1_id, StructFieldData { ty: Type::I32 });
                        field.insert(f2_id, StructFieldData { ty: Type::F32 });

                        field
                    },
                }),
            });
        }
    }

    let s1 = table.lookup_type(&Type::Custom(s1_id.to_string())).unwrap();
    assert_eq!(s1.members.len(), 2);
    assert!(s1.members.get(&f1_id).is_some());
    assert!(s1.members.get(&f2_id).is_some());
    assert!(s1.members.get(&m1_id).is_none());

    assert!(table
        .lookup_type(&Type::Custom(m2_id.to_string()))
        .is_none());
}
