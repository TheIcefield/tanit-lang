pub(crate) mod entry;
pub(crate) mod table;
pub(crate) mod type_info;

#[cfg(test)]
mod tests {
    use tanitc_attributes::{Mutability, Safety};
    use tanitc_hir::hir::type_spec::{FuncType, Type};
    use tanitc_ident::Ident;
    use tanitc_lexer::location::Location;
    use tanitc_name::NameSpec;

    use crate::symbol_table::{
        entry::{Entry, StructFieldsData, SymbolKind},
        table::{ScopeInfo, Table},
    };

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

        use crate::symbol_table::entry::{FuncDefData, ModuleDefData, VarDefData, VarStorageType};

        let main_mod_id = Ident::from("Main".to_string());
        let main_fn_id = Ident::from("main".to_string());
        let bar_id = Ident::from("bar".to_string());
        let var_id = Ident::from("var".to_string());
        let baz_id = Ident::from("baz".to_string());

        let mut table = Table::new();

        let mut name_spec = NameSpec {
            location: Location::default(),
            path: vec![],
        };

        table.insert(Entry {
            id: main_mod_id,
            is_static: true,
            kind: ModuleDefData {
                name: {
                    let mut name = name_spec.clone();
                    name.path.push(main_mod_id.into());

                    name
                },
                table: Box::new(Table::new()),
            }
            .into(),
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

            name_spec.path = vec![bar_id.into()];

            table.insert(Entry {
                id: bar_id,
                is_static: false,
                kind: SymbolKind::from(FuncDefData {
                    name: name_spec.clone(),
                    ty: FuncType {
                        parameters: vec![],
                        return_type: Box::new(Type::unit()),
                        safety: Safety::Safe,
                    },
                    is_virtual: false,
                    is_inline: false,
                    no_return: true,
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

            name_spec.path = table.get_joined_path(main_fn_id);

            table.insert(Entry {
                id: main_fn_id,
                is_static: false,
                kind: SymbolKind::from(FuncDefData {
                    name: name_spec.clone(),
                    ty: FuncType {
                        parameters: vec![],
                        return_type: Box::new(Type::unit()),
                        safety: Safety::Safe,
                    },
                    is_virtual: false,
                    is_inline: false,
                    no_return: true,
                }),
            });

            {
                table.enter_scope(ScopeInfo {
                    safety: Safety::Safe,
                    is_in_func: true,
                    ..Default::default()
                }); // enter main

                table.insert(Entry {
                    id: var_id,
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

        use crate::symbol_table::entry::{FuncDefData, ModuleDefData};

        // Given
        let m1_id = Ident::from("M1".to_string());
        let m2_id = Ident::from("M2".to_string());
        let f1_id = Ident::from("f1".to_string());
        let f2_id = Ident::from("f2".to_string());

        let mut table = Table::new();
        let mut name_spec = NameSpec {
            location: Location::default(),
            path: vec![],
        };

        // INSERT M1
        table.insert(Entry {
            id: m1_id,
            is_static: true,
            kind: ModuleDefData {
                name: {
                    let mut name = name_spec.clone();
                    name.path.push(m1_id.into());
                    name
                },
                table: {
                    // INSERT FUNCTIONS INTO M1
                    let mut table = Box::new(Table::new());
                    let mut name_spec = name_spec.clone();

                    table.set_path(vec![m1_id.into()]);
                    name_spec.path = table.get_joined_path(f1_id);

                    table.insert(Entry {
                        id: f1_id,
                        is_static: false,
                        kind: FuncDefData {
                            name: name_spec.clone(),
                            ty: FuncType {
                                parameters: vec![],
                                return_type: Box::new(Type::unit()),
                                safety: Safety::Safe,
                            },
                            is_virtual: false,
                            is_inline: false,
                            no_return: true,
                        }
                        .into(),
                    });

                    name_spec.path = table.get_joined_path(f2_id);

                    table.insert(Entry {
                        id: f2_id,
                        is_static: false,
                        kind: FuncDefData {
                            name: name_spec.clone(),
                            ty: FuncType {
                                parameters: vec![],
                                return_type: Box::new(Type::unit()),
                                safety: Safety::Safe,
                            },
                            is_virtual: false,
                            is_inline: false,
                            no_return: true,
                        }
                        .into(),
                    });

                    table
                },
            }
            .into(),
        });

        // INSERT M2
        table.insert(Entry {
            id: m2_id,
            is_static: true,
            kind: ModuleDefData {
                name: {
                    let mut name = name_spec.clone();
                    name.path.push(m2_id.into());
                    name
                },
                table: {
                    let mut table = Box::new(Table::new());
                    let mut name_spec = name_spec.clone();

                    table.set_path(vec![m2_id.into()]);
                    name_spec.path = table.get_joined_path(f2_id);

                    table.insert(Entry {
                        id: f2_id,
                        is_static: false,
                        kind: FuncDefData {
                            name: name_spec.clone(),
                            ty: FuncType {
                                parameters: vec![],
                                return_type: Box::new(Type::unit()),
                                safety: Safety::Safe,
                            },
                            is_virtual: false,
                            is_inline: false,
                            no_return: true,
                        }
                        .into(),
                    });

                    table
                },
            }
            .into(),
        });

        // Then
        name_spec.path = vec![f1_id.into()];
        assert!(table.lookup_name_spec(&name_spec).is_err());

        name_spec.path = vec![f2_id.into()];
        assert!(table.lookup_name_spec(&name_spec).is_err());

        name_spec.path = vec![m1_id.into()];
        assert!(table.lookup_name_spec(&name_spec).is_ok());

        name_spec.path = vec![m2_id.into()];
        assert!(table.lookup_name_spec(&name_spec).is_ok());

        name_spec.path = vec![m1_id.into(), f1_id.into()];
        assert!(table.lookup_name_spec(&name_spec).is_ok());

        name_spec.path = vec![m1_id.into(), f2_id.into()];
        assert!(table.lookup_name_spec(&name_spec).is_ok());

        name_spec.path = vec![m2_id.into(), f1_id.into()];
        assert!(table.lookup_name_spec(&name_spec).is_err());

        name_spec.path = vec![m2_id.into(), f2_id.into()];
        assert!(table.lookup_name_spec(&name_spec).is_ok());
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

        use crate::symbol_table::entry::{ModuleDefData, StructDefData, StructFieldData};

        let m1_id = Ident::from("M1".to_string());
        let m2_id = Ident::from("M2".to_string());
        let s1_id = Ident::from("S1".to_string());
        let f1_id = Ident::from("f1".to_string());
        let f2_id = Ident::from("f2".to_string());

        let mut table = Table::new();

        let mut name_spec = NameSpec {
            location: Location::default(),
            path: vec![],
        };

        table.insert(Entry {
            id: m1_id,
            is_static: true,
            kind: ModuleDefData {
                name: {
                    name_spec.path.push(m1_id.into());
                    name_spec.clone()
                },
                table: Box::new(Table::new()),
            }
            .into(),
        });

        {
            let m1 = table.lookup_mut(m1_id).unwrap();
            let SymbolKind::ModuleDef(ref mut data) = &mut m1.kind else {
                unreachable!()
            };

            data.table.insert(Entry {
                id: m2_id,
                is_static: true,
                kind: ModuleDefData {
                    name: {
                        name_spec.path.push(m2_id.into());
                        name_spec.clone()
                    },
                    table: Box::new(Table::new()),
                }
                .into(),
            });

            {
                let mut prefix = data.table.get_joined_path(m2_id);

                let m2 = data.table.lookup_mut(m2_id).unwrap();
                let SymbolKind::ModuleDef(ref mut data) = &mut m2.kind else {
                    unreachable!()
                };

                name_spec.path.append(&mut prefix);
                name_spec.path.push(s1_id.into());

                data.table.insert(Entry {
                    id: s1_id,
                    is_static: false,
                    kind: StructDefData {
                        name: name_spec.clone(),
                        fields: {
                            let mut field = StructFieldsData::new();

                            field.insert(
                                f1_id,
                                StructFieldData {
                                    name: name_spec.clone(),
                                    ty: Type::I32,
                                },
                            );
                            field.insert(
                                f2_id,
                                StructFieldData {
                                    name: name_spec.clone(),
                                    ty: Type::F32,
                                },
                            );

                            field
                        },
                    }
                    .into(),
                });
            }
        }

        name_spec.path = vec![s1_id.into()];
        let s1_ty = Type::Custom(name_spec.clone());
        let s1 = table.lookup_type(&s1_ty).unwrap();
        assert_eq!(s1.members.len(), 2);
        assert!(s1.members.contains_key(&f1_id));
        assert!(s1.members.contains_key(&f2_id));
        assert!(!s1.members.contains_key(&m1_id));

        name_spec.path = vec![m2_id.into()];
        let m2_ty = Type::Custom(name_spec.clone());
        assert!(table.lookup_type(&m2_ty).is_none());
    }
}
