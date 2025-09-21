use tanitc_ast::ast::aliases::AliasDef;
use tanitc_attributes::Mutability;
use tanitc_messages::Message;
use tanitc_symbol_table::{
    entry::{AliasDefData, Entry, SymbolKind},
    type_info::TypeInfo,
};
use tanitc_ty::Type;

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
            kind: SymbolKind::AliasDef(AliasDefData {
                ty: alias_def.value.get_type(),
            }),
        });

        Ok(())
    }

    pub fn get_alias_def_type(&self, alias_def: &AliasDef) -> TypeInfo {
        TypeInfo {
            ty: alias_def.value.get_type(),
            mutability: Mutability::default(),
            ..Default::default()
        }
    }

    pub fn find_alias_value(&self, alias_type: &Type) -> Option<Type> {
        let Type::Custom(type_id) = alias_type else {
            return None;
        };

        let entry = self.table.lookup(type_id.id)?;

        let SymbolKind::AliasDef(alias_data) = &entry.kind else {
            return None;
        };

        let Some(alias_to) = self.find_alias_value(&alias_data.ty) else {
            return Some(alias_data.ty.clone());
        };

        Some(alias_to)
    }
}

#[cfg(test)]
mod tests {
    use tanitc_ast::ast::{
        aliases::AliasDef,
        blocks::Block,
        expressions::{BinaryOperation, Expression, ExpressionKind},
        functions::FunctionDef,
        structs::{StructDef, StructFieldInfo, StructFields},
        types::TypeSpec,
        values::{Value, ValueKind},
        variables::VariableDef,
        Ast,
    };
    use tanitc_ident::{Ident, Name};
    use tanitc_lexer::location::Location;
    use tanitc_ty::Type;

    use crate::Analyzer;

    fn get_alias(name: &str, ty: Type) -> AliasDef {
        AliasDef {
            identifier: Ident::from(name.to_string()),
            value: TypeSpec {
                ty,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn get_struct(name: &str, fields: &[(&str, Type)]) -> StructDef {
        StructDef {
            name: Name::from(name.to_string()),
            fields: {
                let mut local_fields = StructFields::new();
                for (field_name, field_ty) in fields.iter() {
                    local_fields.insert(
                        Ident::from(field_name.to_string()),
                        StructFieldInfo {
                            ty: TypeSpec {
                                ty: field_ty.clone(),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                    );
                }
                local_fields
            },
            ..Default::default()
        }
    }

    fn get_func(name: &str, statements: Vec<Ast>) -> FunctionDef {
        FunctionDef {
            name: Name::from(name.to_string()),
            return_type: TypeSpec {
                ty: Type::unit(),
                ..Default::default()
            },
            body: Some(Box::new(Block {
                is_global: false,
                statements,
                ..Default::default()
            })),
            ..Default::default()
        }
    }

    fn get_var_init(var_name: &str, var_type: Type, obj: ValueKind) -> Expression {
        Expression {
            location: Location::new(),
            kind: ExpressionKind::Binary {
                operation: BinaryOperation::Assign,
                lhs: Box::new(
                    VariableDef {
                        identifier: Ident::from(var_name.to_string()),
                        var_type: TypeSpec {
                            ty: var_type,
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                    .into(),
                ),
                rhs: Box::new(
                    Value {
                        location: Location::new(),
                        kind: obj,
                    }
                    .into(),
                ),
            },
        }
    }

    fn get_struct_init(struct_name: &str, components: Vec<(&str, f64)>) -> ValueKind {
        ValueKind::Struct {
            name: Name::from(struct_name.to_string()),
            components: {
                let mut local_components = Vec::<(Name, Ast)>::with_capacity(components.len());
                for (comp_name, comp_val) in components.iter() {
                    local_components.push((
                        Name::from(comp_name.to_string()),
                        get_float(*comp_val).into(),
                    ));
                }
                local_components
            },
        }
    }

    fn get_float(value: f64) -> Value {
        Value {
            location: Location::new(),
            kind: ValueKind::Decimal(value),
        }
    }

    #[test]
    fn alias_test() {
        /*
         * pub alias VecUnit = f32
         *
         * pub struct Vec2 {
         *     x: VecUnit
         *     y: VecUnit
         * }
         *
         * alias Vec = Vec2
         *
         * func main() {
         *     var v = Vec { x: 10.0, y: 10.0 }
         * }
         */

        const STRUCT_NAME: &str = "Vec2";
        const FIELD_1_NAME: &str = "x";
        const FIELD_2_NAME: &str = "y";
        const FIELD_1_VAL: f64 = 10.0;
        const FIELD_2_VAL: f64 = 10.0;
        const FIRST_ALIAS: &str = "VecUnit";
        const SECOND_ALIAS: &str = "Vec";
        const FUNC_NAME: &str = "main";
        const VAR_NAME: &str = "v";

        let mut program = Ast::from(Block {
            is_global: true,
            statements: vec![
                get_alias(FIRST_ALIAS, Type::F32).into(),
                get_struct(
                    STRUCT_NAME,
                    &[
                        (
                            FIELD_1_NAME,
                            Type::Custom(Name::from(FIRST_ALIAS.to_string())),
                        ),
                        (
                            FIELD_2_NAME,
                            Type::Custom(Name::from(FIRST_ALIAS.to_string())),
                        ),
                    ],
                )
                .into(),
                get_alias(SECOND_ALIAS, Type::Custom(Name::from("Vec2".to_string()))).into(),
                get_func(
                    FUNC_NAME,
                    vec![get_var_init(
                        VAR_NAME,
                        Type::Auto,
                        get_struct_init(
                            SECOND_ALIAS,
                            vec![(FIELD_1_NAME, FIELD_1_VAL), (FIELD_2_NAME, FIELD_2_VAL)],
                        ),
                    )
                    .into()],
                )
                .into(),
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
    fn incorrect_alias_object_test() {
        /*
         * alias Vec = i32
         * func main() {
         *     var v = Vec { x: 10.0, y: 10.0 }
         * };
         */

        const ALIAS_NAME: &str = "Vec";
        const ALIAS_VALUE: Type = Type::I32;
        const FUNC_NAME: &str = "main";
        const VAR_NAME: &str = "v";

        const EXPECTED_ERR: &str = "Semantic error: Common type \"i32\" does not have any fields";

        let mut program = Ast::from(Block {
            is_global: true,
            statements: vec![
                get_alias(ALIAS_NAME, ALIAS_VALUE).into(),
                get_func(
                    FUNC_NAME,
                    vec![get_var_init(
                        VAR_NAME,
                        Type::Auto,
                        get_struct_init(ALIAS_NAME, vec![("x", 10.0), ("y", 10.0)]),
                    )
                    .into()],
                )
                .into(),
            ],
            ..Default::default()
        });

        let mut analyzer = Analyzer::new();
        program.accept_mut(&mut analyzer).unwrap();

        let errors = analyzer.get_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].text, EXPECTED_ERR);
    }

    #[test]
    fn alias_common_type_test() {
        /*
         * alias A = i32
         * func main() {
         *     var a: A = 100
         * }
         */

        const ALIAS_NAME: &str = "A";
        const ALIAS_VALUE: Type = Type::I32;
        const FUNC_NAME: &str = "main";
        const VAR_NAME: &str = "a";
        const VAR_VALUE: usize = 100;

        let mut program = Ast::from(Block {
            is_global: true,
            statements: vec![
                get_alias(ALIAS_NAME, ALIAS_VALUE).into(),
                get_func(
                    FUNC_NAME,
                    vec![get_var_init(
                        VAR_NAME,
                        Type::Custom(Name::from(ALIAS_NAME.to_string())),
                        ValueKind::Integer(VAR_VALUE),
                    )
                    .into()],
                )
                .into(),
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
    fn incorrect_alias_common_type_test() {
        /*
         * alias A = i32
         * func main() {
         *     var a: A = 3.14
         * }
         */

        const ALIAS_NAME: &str = "A";
        const ALIAS_VALUE: Type = Type::I32;
        const FUNC_NAME: &str = "main";
        const VAR_NAME: &str = "a";
        const VAR_VALUE: f64 = 3.14;

        const EXPECTED_ERR: &str = "Semantic error: Cannot perform operation on objects with different types: A (aka: i32) and f32";

        let mut program = Ast::from(Block {
            is_global: true,
            statements: vec![
                get_alias(ALIAS_NAME, ALIAS_VALUE).into(),
                get_func(
                    FUNC_NAME,
                    vec![get_var_init(
                        VAR_NAME,
                        Type::Custom(Name::from(ALIAS_NAME.to_string())),
                        ValueKind::Decimal(VAR_VALUE),
                    )
                    .into()],
                )
                .into(),
            ],
            ..Default::default()
        });

        let mut analyzer = Analyzer::new();

        program.accept_mut(&mut analyzer).unwrap();
        let errors = analyzer.get_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].text, EXPECTED_ERR);
    }

    #[test]
    fn alias_custom_type_test() {
        /*
         * struct S {}
         * alias A = S
         * func main() {
         *     var a: A = S {}
         * }";
         */

        const ALIAS_NAME: &str = "A";
        const STRUCT_NAME: &str = "S";
        const FUNC_NAME: &str = "main";
        const VAR_NAME: &str = "a";

        let mut program = Ast::from(Block {
            is_global: true,
            statements: vec![
                get_struct(STRUCT_NAME, &[]).into(),
                get_alias(
                    ALIAS_NAME,
                    Type::Custom(Name::from(STRUCT_NAME.to_string())),
                )
                .into(),
                get_func(
                    FUNC_NAME,
                    vec![get_var_init(
                        VAR_NAME,
                        Type::Custom(Name::from(ALIAS_NAME.to_string())),
                        get_struct_init(STRUCT_NAME, vec![]),
                    )
                    .into()],
                )
                .into(),
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
    fn incorrect_alias_custom_type_test() {
        /*
         * struct S {}
         * alias A = S
         * func main() {
         *     var a: A = 100
         * }";
         */

        const ALIAS_NAME: &str = "A";
        const STRUCT_NAME: &str = "S";
        const FUNC_NAME: &str = "main";
        const VAR_NAME: &str = "a";

        const EXPECTED_ERR: &str = "Semantic error: Cannot perform operation on objects with different types: A (aka: S) and i32";

        let mut program = Ast::from(Block {
            is_global: true,
            statements: vec![
                get_struct(STRUCT_NAME, &[]).into(),
                get_alias(
                    ALIAS_NAME,
                    Type::Custom(Name::from(STRUCT_NAME.to_string())),
                )
                .into(),
                get_func(
                    FUNC_NAME,
                    vec![get_var_init(
                        VAR_NAME,
                        Type::Custom(Name::from(ALIAS_NAME.to_string())),
                        ValueKind::Integer(100),
                    )
                    .into()],
                )
                .into(),
            ],
            ..Default::default()
        });

        let mut analyzer = Analyzer::new();
        program.accept_mut(&mut analyzer).unwrap();

        let errors = analyzer.get_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].text, EXPECTED_ERR);
    }

    #[test]
    fn alias_to_alias_type_test() {
        /*
         * struct S {}
         * alias A = S
         * alias B = A
         *
         * func main() {
         *     var b: B = S {}
         * }
         */

        const ALIAS_1_NAME: &str = "A";
        const ALIAS_2_NAME: &str = "B";
        const STRUCT_NAME: &str = "S";
        const FUNC_NAME: &str = "main";
        const VAR_NAME: &str = "b";

        let mut program = Ast::from(Block {
            is_global: true,
            statements: vec![
                get_struct(STRUCT_NAME, &[]).into(),
                get_alias(
                    ALIAS_1_NAME,
                    Type::Custom(Name::from(STRUCT_NAME.to_string())),
                )
                .into(),
                get_alias(
                    ALIAS_2_NAME,
                    Type::Custom(Name::from(ALIAS_1_NAME.to_string())),
                )
                .into(),
                get_func(
                    FUNC_NAME,
                    vec![get_var_init(
                        VAR_NAME,
                        Type::Custom(Name::from(ALIAS_2_NAME.to_string())),
                        get_struct_init(STRUCT_NAME, vec![]),
                    )
                    .into()],
                )
                .into(),
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
    fn incorrect_alias_to_alias_type_test() {
        /*
         * struct S {}
         * alias A = S
         * alias B = A
         * func main() {
         *     var b: B = 50
         * }
         */

        const ALIAS_1_NAME: &str = "A";
        const ALIAS_2_NAME: &str = "B";
        const STRUCT_NAME: &str = "S";
        const FUNC_NAME: &str = "main";
        const VAR_NAME: &str = "b";

        const EXPECTED_ERR: &str = "Semantic error: Cannot perform operation on objects with different types: B (aka: S) and i32";

        let mut program = Ast::from(Block {
            is_global: true,
            statements: vec![
                get_struct(STRUCT_NAME, &[]).into(),
                get_alias(
                    ALIAS_1_NAME,
                    Type::Custom(Name::from(STRUCT_NAME.to_string())),
                )
                .into(),
                get_alias(
                    ALIAS_2_NAME,
                    Type::Custom(Name::from(ALIAS_1_NAME.to_string())),
                )
                .into(),
                get_func(
                    FUNC_NAME,
                    vec![get_var_init(
                        VAR_NAME,
                        Type::Custom(Name::from(ALIAS_2_NAME.to_string())),
                        ValueKind::Integer(50),
                    )
                    .into()],
                )
                .into(),
            ],
            ..Default::default()
        });

        let mut analyzer = Analyzer::new();
        program.accept_mut(&mut analyzer).unwrap();

        let errors = analyzer.get_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].text, EXPECTED_ERR);
    }
}
