use tanitc_ast::AliasDef;
use tanitc_messages::Message;
use tanitc_symbol_table::entry::{AliasDefData, Entry, SymbolKind};

use crate::Analyzer;

impl Analyzer {
    pub fn analyze_alias_def(&mut self, alias_def: &mut AliasDef) -> Result<(), Message> {
        if self.has_symbol(alias_def.identifier) {
            return Err(Message::multiple_ids(
                alias_def.location,
                alias_def.identifier,
            ));
        }

        self.add_symbol(Entry {
            name: alias_def.identifier,
            is_static: true,
            kind: SymbolKind::from(AliasDefData {
                ty: alias_def.value.get_type(),
            }),
        });

        Ok(())
    }
}

#[cfg(test)]
mod alias_test {
    use crate::Analyzer;

    use tanitc_ast::{
        expression_utils::BinaryOperation, AliasDef, Ast, Block, Expression, ExpressionKind,
        FieldInfo, Fields, FunctionDef, StructDef, TypeSpec, Value, ValueKind, VariableDef,
    };
    use tanitc_ident::Ident;
    use tanitc_lexer::location::Location;
    use tanitc_ty::Type;

    #[test]
    fn alias_good_test() {
        /* alias VecUnit = f32
         * struct Vec2 {
         *     x: VecUnit
         *     y: VecUnit
         * }
         * alias Vec = Vec2
         * func foo() {
         *     var v = Vec { x: 10.0, y: 10.0 }
         * }
         */

        let vec_id = Ident::from("Vec".to_string());
        let vec2_id = Ident::from("Vec2".to_string());
        let x_id = Ident::from("x".to_string());
        let y_id = Ident::from("y".to_string());
        let v_id = Ident::from("v".to_string());
        let vec_unit_id = Ident::from("VecUnit".to_string());
        let foo_id = Ident::from("foo".to_string());

        let vec_unit_alias = AliasDef {
            identifier: vec_unit_id,
            value: TypeSpec {
                ty: Type::F32,
                ..Default::default()
            },
            ..Default::default()
        };
        let vec_struct = StructDef {
            identifier: vec2_id,
            fields: {
                let mut fields = Fields::new();
                let field_info = FieldInfo {
                    ty: TypeSpec {
                        ty: Type::Custom(vec_unit_id),
                        ..Default::default()
                    },
                    ..Default::default()
                };
                fields.insert(x_id, field_info.clone());
                fields.insert(y_id, field_info);
                fields
            },
            ..Default::default()
        };
        let vec_alias = AliasDef {
            identifier: vec_id,
            value: TypeSpec {
                ty: Type::Custom(vec2_id),
                ..Default::default()
            },
            ..Default::default()
        };
        let var_def = VariableDef {
            is_global: false,
            is_mutable: false,
            identifier: v_id,
            var_type: TypeSpec {
                ty: Type::Auto,
                ..Default::default()
            },
            ..Default::default()
        };
        let struct_init = Value {
            kind: ValueKind::Struct {
                identifier: vec_id,
                components: vec![
                    (
                        x_id,
                        Value {
                            kind: ValueKind::Decimal(10.0),
                            location: Location::default(),
                        }
                        .into(),
                    ),
                    (
                        y_id,
                        Value {
                            kind: ValueKind::Decimal(10.0),
                            location: Location::default(),
                        }
                        .into(),
                    ),
                ],
            },
            location: Location::default(),
        };
        let function = FunctionDef {
            identifier: foo_id,
            body: Some(Box::new(
                Block {
                    is_global: false,
                    statements: vec![Expression {
                        kind: ExpressionKind::Binary {
                            operation: BinaryOperation::Assign,
                            lhs: Box::new(var_def.into()),
                            rhs: Box::new(struct_init.into()),
                        },
                        location: Location::default(),
                    }
                    .into()],
                    ..Default::default()
                }
                .into(),
            )),
            ..Default::default()
        };

        let mut program = Ast::from(Block {
            is_global: true,
            statements: vec![
                vec_unit_alias.into(),
                vec_struct.into(),
                vec_alias.into(),
                function.into(),
            ],
            ..Default::default()
        });

        let mut analyzer = Analyzer::new();
        program.accept_mut(&mut analyzer).unwrap();
        if analyzer.has_errors() {
            panic!("{:#?}", analyzer.get_errors());
        }
    }

    #[test]
    fn alias_object_bad_test() {
        /*
         * alias Unit = i32
         * func foo() {
         *     var v = Unit { value: 10 }
         * }
         */

        const EXPECTED_ERR: &str = "Semantic error: Common type \"i32\" does not have any fields";

        let unit_id = Ident::from("Unit".to_string());
        let value_id = Ident::from("value".to_string());
        let v_id = Ident::from("v".to_string());
        let foo_id = Ident::from("foo".to_string());

        let vec_unit_alias = AliasDef {
            identifier: unit_id,
            value: TypeSpec {
                ty: Type::I32,
                ..Default::default()
            },
            ..Default::default()
        };
        let var_def = VariableDef {
            is_global: false,
            is_mutable: false,
            identifier: v_id,
            var_type: TypeSpec {
                ty: Type::Auto,
                ..Default::default()
            },
            ..Default::default()
        };
        let struct_init = Value {
            kind: ValueKind::Struct {
                identifier: unit_id,
                components: vec![(
                    value_id,
                    Value {
                        kind: ValueKind::Decimal(10.0),
                        location: Location::default(),
                    }
                    .into(),
                )],
            },
            location: Location::default(),
        };
        let function = FunctionDef {
            identifier: foo_id,
            body: Some(Box::new(
                Block {
                    is_global: false,
                    statements: vec![Expression {
                        kind: ExpressionKind::Binary {
                            operation: BinaryOperation::Assign,
                            lhs: Box::new(var_def.into()),
                            rhs: Box::new(struct_init.into()),
                        },
                        location: Location::default(),
                    }
                    .into()],
                    ..Default::default()
                }
                .into(),
            )),
            ..Default::default()
        };

        let mut program = Ast::from(Block {
            is_global: true,
            statements: vec![vec_unit_alias.into(), function.into()],
            ..Default::default()
        });

        let mut analyzer = Analyzer::new();
        program.accept_mut(&mut analyzer).unwrap();

        let errors = analyzer.get_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].text, EXPECTED_ERR);
    }

    #[test]
    fn alias_common_type_good_test() {
        /*
         * alias Unit = i32
         * func foo() {
         *     var v: Unit = 10
         * }
         */

        let unit_id = Ident::from("Unit".to_string());
        let v_id = Ident::from("v".to_string());
        let foo_id = Ident::from("foo".to_string());

        let unit_alias = AliasDef {
            identifier: unit_id,
            value: TypeSpec {
                ty: Type::I32,
                ..Default::default()
            },
            ..Default::default()
        };
        let var_def = VariableDef {
            is_global: false,
            is_mutable: false,
            identifier: v_id,
            var_type: TypeSpec {
                ty: Type::Custom(unit_id),
                ..Default::default()
            },
            ..Default::default()
        };
        let var_init = Value {
            kind: ValueKind::Integer(10),
            location: Location::default(),
        };
        let function = FunctionDef {
            identifier: foo_id,
            body: Some(Box::new(
                Block {
                    is_global: false,
                    statements: vec![Expression {
                        kind: ExpressionKind::Binary {
                            operation: BinaryOperation::Assign,
                            lhs: Box::new(var_def.into()),
                            rhs: Box::new(var_init.into()),
                        },
                        location: Location::default(),
                    }
                    .into()],
                    ..Default::default()
                }
                .into(),
            )),
            ..Default::default()
        };

        let mut program = Ast::from(Block {
            is_global: true,
            statements: vec![unit_alias.into(), function.into()],
            ..Default::default()
        });

        let mut analyzer = Analyzer::new();
        program.accept_mut(&mut analyzer).unwrap();
        if analyzer.has_errors() {
            panic!("{:#?}", analyzer.get_errors());
        }
    }

    #[test]
    fn alias_common_type_bad_test() {
        /*
         * alias Unit = i32
         * func foo() {
         *     var v: Unit = 10.0
         * }
         */

        const EXPECTED_ERR: &str = "Semantic error: Cannot perform operation on objects with different types: Unit (aka: i32) and f32";

        let unit_id = Ident::from("Unit".to_string());
        let v_id = Ident::from("v".to_string());
        let foo_id = Ident::from("foo".to_string());

        let unit_alias = AliasDef {
            identifier: unit_id,
            value: TypeSpec {
                ty: Type::I32,
                ..Default::default()
            },
            ..Default::default()
        };
        let var_def = VariableDef {
            is_global: false,
            is_mutable: false,
            identifier: v_id,
            var_type: TypeSpec {
                ty: Type::Custom(unit_id),
                ..Default::default()
            },
            ..Default::default()
        };
        let var_init = Value {
            kind: ValueKind::Decimal(10.0),
            location: Location::default(),
        };
        let function = FunctionDef {
            identifier: foo_id,
            body: Some(Box::new(
                Block {
                    is_global: false,
                    statements: vec![Expression {
                        kind: ExpressionKind::Binary {
                            operation: BinaryOperation::Assign,
                            lhs: Box::new(var_def.into()),
                            rhs: Box::new(var_init.into()),
                        },
                        location: Location::default(),
                    }
                    .into()],
                    ..Default::default()
                }
                .into(),
            )),
            ..Default::default()
        };

        let mut program = Ast::from(Block {
            is_global: true,
            statements: vec![unit_alias.into(), function.into()],
            ..Default::default()
        });

        let mut analyzer = Analyzer::new();
        program.accept_mut(&mut analyzer).unwrap();

        let errors = analyzer.get_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].text, EXPECTED_ERR);
    }

    #[test]
    fn alias_custom_type_good_test() {
        /*
         * alias Unit = i32
         * func foo() {
         *     var v: Unit = 10
         * }
         */

        let unit_id = Ident::from("Unit".to_string());
        let v_id = Ident::from("v".to_string());
        let foo_id = Ident::from("foo".to_string());

        let unit_alias = AliasDef {
            identifier: unit_id,
            value: TypeSpec {
                ty: Type::I32,
                ..Default::default()
            },
            ..Default::default()
        };
        let var_def = VariableDef {
            is_global: false,
            is_mutable: false,
            identifier: v_id,
            var_type: TypeSpec {
                ty: Type::Custom(unit_id),
                ..Default::default()
            },
            ..Default::default()
        };
        let var_init = Value {
            kind: ValueKind::Integer(10),
            location: Location::default(),
        };
        let function = FunctionDef {
            identifier: foo_id,
            body: Some(Box::new(
                Block {
                    is_global: false,
                    statements: vec![Expression {
                        kind: ExpressionKind::Binary {
                            operation: BinaryOperation::Assign,
                            lhs: Box::new(var_def.into()),
                            rhs: Box::new(var_init.into()),
                        },
                        location: Location::default(),
                    }
                    .into()],
                    ..Default::default()
                }
                .into(),
            )),
            ..Default::default()
        };

        let mut program = Ast::from(Block {
            is_global: true,
            statements: vec![unit_alias.into(), function.into()],
            ..Default::default()
        });

        let mut analyzer = Analyzer::new();
        program.accept_mut(&mut analyzer).unwrap();
        if analyzer.has_errors() {
            panic!("{:#?}", analyzer.get_errors());
        }
    }

    #[test]
    fn alias_custom_type_bad_test() {
        /*
         * struct S { }
         * alias Unit = S
         * func foo() {
         *     var v: Unit = 10.0
         * }
         */

        const EXPECTED_ERR: &str = "Semantic error: Cannot perform operation on objects with different types: Unit (aka: S) and f32";

        let s_id = Ident::from("S".to_string());
        let unit_id = Ident::from("Unit".to_string());
        let v_id = Ident::from("v".to_string());
        let foo_id = Ident::from("foo".to_string());

        let struct_def = StructDef {
            identifier: s_id,
            ..Default::default()
        };
        let unit_alias = AliasDef {
            identifier: unit_id,
            value: TypeSpec {
                ty: Type::Custom(s_id),
                ..Default::default()
            },
            ..Default::default()
        };
        let var_def = VariableDef {
            is_global: false,
            is_mutable: false,
            identifier: v_id,
            var_type: TypeSpec {
                ty: Type::Custom(unit_id),
                ..Default::default()
            },
            ..Default::default()
        };
        let var_init = Value {
            kind: ValueKind::Decimal(10.0),
            location: Location::default(),
        };
        let function = FunctionDef {
            identifier: foo_id,
            body: Some(Box::new(
                Block {
                    is_global: false,
                    statements: vec![Expression {
                        kind: ExpressionKind::Binary {
                            operation: BinaryOperation::Assign,
                            lhs: Box::new(var_def.into()),
                            rhs: Box::new(var_init.into()),
                        },
                        location: Location::default(),
                    }
                    .into()],
                    ..Default::default()
                }
                .into(),
            )),
            ..Default::default()
        };

        let mut program = Ast::from(Block {
            is_global: true,
            statements: vec![struct_def.into(), unit_alias.into(), function.into()],
            ..Default::default()
        });

        let mut analyzer = Analyzer::new();
        program.accept_mut(&mut analyzer).unwrap();

        let errors = analyzer.get_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].text, EXPECTED_ERR);
    }

    #[test]
    fn alias_to_alias_type_good_test() {
        /*
         * struct S {}
         * alias A = S
         * alias B = A
         * func foo() {
         *     var v: B = S {}
         * }
         */

        let s_id = Ident::from("S".to_string());
        let a_id = Ident::from("A".to_string());
        let b_id = Ident::from("B".to_string());
        let v_id = Ident::from("v".to_string());
        let foo_id = Ident::from("foo".to_string());

        let struct_def = StructDef {
            identifier: s_id,
            ..Default::default()
        };
        let a_alias_def = AliasDef {
            identifier: a_id,
            value: TypeSpec {
                ty: Type::Custom(s_id),
                ..Default::default()
            },
            ..Default::default()
        };
        let b_alias_def = AliasDef {
            identifier: b_id,
            value: TypeSpec {
                ty: Type::Custom(a_id),
                ..Default::default()
            },
            ..Default::default()
        };
        let var_def = VariableDef {
            is_global: false,
            is_mutable: false,
            identifier: v_id,
            var_type: TypeSpec {
                ty: Type::Custom(b_id),
                ..Default::default()
            },
            ..Default::default()
        };
        let var_init = Value {
            kind: ValueKind::Struct {
                identifier: s_id,
                components: vec![],
            },
            location: Location::default(),
        };
        let function = FunctionDef {
            identifier: foo_id,
            body: Some(Box::new(
                Block {
                    is_global: false,
                    statements: vec![Expression {
                        kind: ExpressionKind::Binary {
                            operation: BinaryOperation::Assign,
                            lhs: Box::new(var_def.into()),
                            rhs: Box::new(var_init.into()),
                        },
                        location: Location::default(),
                    }
                    .into()],
                    ..Default::default()
                }
                .into(),
            )),
            ..Default::default()
        };

        let mut program = Ast::from(Block {
            is_global: true,
            statements: vec![
                struct_def.into(),
                a_alias_def.into(),
                b_alias_def.into(),
                function.into(),
            ],
            ..Default::default()
        });

        let mut analyzer = Analyzer::new();
        program.accept_mut(&mut analyzer).unwrap();
        if analyzer.has_errors() {
            panic!("{:#?}", analyzer.get_errors());
        }
    }

    #[test]
    fn alias_to_alias_type_bad_test() {
        /*
         * struct S {}
         * alias A = S
         * alias B = A
         * func foo() {
         *     var v: B = 10
         * }
         */

        const EXPECTED: &str = "Semantic error: Cannot perform operation on objects with different types: B (aka: S) and i32";

        let s_id = Ident::from("S".to_string());
        let a_id = Ident::from("A".to_string());
        let b_id = Ident::from("B".to_string());
        let v_id = Ident::from("v".to_string());
        let foo_id = Ident::from("foo".to_string());

        let struct_def = StructDef {
            identifier: s_id,
            ..Default::default()
        };
        let a_alias_def = AliasDef {
            identifier: a_id,
            value: TypeSpec {
                ty: Type::Custom(s_id),
                ..Default::default()
            },
            ..Default::default()
        };
        let b_alias_def = AliasDef {
            identifier: b_id,
            value: TypeSpec {
                ty: Type::Custom(a_id),
                ..Default::default()
            },
            ..Default::default()
        };
        let var_def = VariableDef {
            is_global: false,
            is_mutable: false,
            identifier: v_id,
            var_type: TypeSpec {
                ty: Type::Custom(b_id),
                ..Default::default()
            },
            ..Default::default()
        };
        let var_init = Value {
            kind: ValueKind::Integer(10),
            location: Location::default(),
        };
        let function = FunctionDef {
            identifier: foo_id,
            body: Some(Box::new(
                Block {
                    is_global: false,
                    statements: vec![Expression {
                        kind: ExpressionKind::Binary {
                            operation: BinaryOperation::Assign,
                            lhs: Box::new(var_def.into()),
                            rhs: Box::new(var_init.into()),
                        },
                        location: Location::default(),
                    }
                    .into()],
                    ..Default::default()
                }
                .into(),
            )),
            ..Default::default()
        };

        let mut program = Ast::from(Block {
            is_global: true,
            statements: vec![
                struct_def.into(),
                a_alias_def.into(),
                b_alias_def.into(),
                function.into(),
            ],
            ..Default::default()
        });

        let mut analyzer = Analyzer::new();
        program.accept_mut(&mut analyzer).unwrap();
        let errors = analyzer.get_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].text, EXPECTED);
    }
}
