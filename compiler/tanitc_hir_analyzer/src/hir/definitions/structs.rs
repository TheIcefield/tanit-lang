use std::collections::BTreeMap;

use tanitc_hir::hir::{
    definitions::structs::StructDef,
    expressions::{
        literal::{Literal, StructLiteral},
        Expression,
    },
};
use tanitc_ident::Ident;
use tanitc_messages::Message;

use crate::{
    symbol_table::entry::{Entry, StructDefData, StructFieldData, SymbolKind},
    AnalyzeResult, Analyzer,
};

impl Analyzer {
    pub(crate) fn analyze_struct_def(&mut self, struct_def: &mut StructDef) -> AnalyzeResult<()> {
        if self.has_symbol(struct_def.name.id) {
            return Err(Message::multiple_ids(
                struct_def.location,
                struct_def.name.id,
            ));
        }

        struct_def.name.prefix = self.table.get_id();

        for internal in struct_def.internals.iter_mut() {
            internal.accept_mut(self)?;
        }

        let mut fields = BTreeMap::<Ident, StructFieldData>::new();
        for (field_id, field_info) in struct_def.fields.iter() {
            let Some(ty) = self.table.lookup_type(&field_info.ty.ty) else {
                self.error(Message::undefined_type(
                    field_info.ty.location,
                    field_info.ty.ty.to_string(),
                ));
                continue;
            };

            fields.insert(
                *field_id,
                StructFieldData {
                    struct_name: struct_def.name,
                    ty: ty.ty,
                },
            );
        }

        self.add_symbol(Entry {
            name: struct_def.name.id,
            is_static: true,
            kind: SymbolKind::from(StructDefData {
                name: struct_def.name,
                fields,
            }),
        });

        Ok(())
    }

    pub(crate) fn access_struct_def(
        &mut self,
        struct_data: &StructDefData,
        node: &mut Expression,
    ) -> AnalyzeResult<()> {
        let location = node.location();

        let Expression::Literal(Literal::Struct(StructLiteral {
            id: struct_name,
            fields: value_comps,
            ..
        })) = node
        else {
            return Err(Message::unreachable(
                node.location(),
                format!("expected StructLiteral, actually: {node:?}"),
            ));
        };

        self.check_struct_literal_components(value_comps, struct_data, location)?;

        *struct_name = struct_data.name;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tanitc_attributes::Mutability;
    use tanitc_hir::hir::{
        types::{ArraySize, Type},
        Hir,
    };
    use tanitc_hir_test::{
        create_array_lit, create_block, create_decimal_lit, create_integer_lit,
        create_main_func_def, create_struct_def, create_struct_lit, create_var_def,
    };

    #[test]
    fn struct_work_test() {
        // Given
        const STRUCT_NAME: &str = "MyStruct";
        const STRUCT_FIELD_1_NAME: &str = "f1";
        const STRUCT_FIELD_1_TYPE: Type = Type::I32;
        const STRUCT_FIELD_2_NAME: &str = "f2";

        let struct_field_2_type = Type::Array {
            size: ArraySize::Fixed(2),
            value_type: Box::new(Type::F32),
        };
        let struct_def = create_struct_def(
            STRUCT_NAME,
            vec![
                (STRUCT_FIELD_1_NAME, STRUCT_FIELD_1_TYPE),
                (STRUCT_FIELD_2_NAME, struct_field_2_type),
            ],
        );

        const VAR_NAME: &str = "var_def";
        let field_1_val = create_integer_lit(1);
        let field_2_val = create_array_lit(vec![create_decimal_lit(2.0), create_decimal_lit(3.0)]);
        let var_def_vec = create_var_def(
            VAR_NAME,
            Mutability::Mutable,
            Type::Auto,
            Some(create_struct_lit(
                STRUCT_NAME,
                &[
                    (STRUCT_FIELD_1_NAME, field_1_val),
                    (STRUCT_FIELD_2_NAME, field_2_val),
                ],
            )),
        );

        let main_func = create_main_func_def(vec![var_def_vec.into()]);

        /*
         * struct MyStruct
         * {
         *     f1: i32
         *     f2: [f32: 2]
         * }
         *
         * func main() {
         *     var mut var_def = MyStruct {
         *         f1: 1,
         *         f2: [2.0, 3.0]
         *     }
         *     s.f1 = 2
         * }";
         */
        let mut program = Hir::from(create_block(vec![struct_def.into(), main_func.into()]));

        let mut analyzer = Analyzer::new();

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        res.expect("Expected no errors");
    }

    /*
    #[test]
    fn struct_in_module_work_test() {
        // Given
        const STRUCT_NAME: &str = "Vector2";
        const STRUCT_FIELD_X_NAME: &str = "x";
        const STRUCT_FIELD_X_TYPE: Type = Type::F32;
        const STRUCT_FIELD_Y_NAME: &str = "y";
        const STRUCT_FIELD_Y_TYPE: Type = Type::F32;
        let vector_2_struct = create_struct_def(
            STRUCT_NAME,
            &[
                (STRUCT_FIELD_X_NAME, STRUCT_FIELD_X_TYPE),
                (STRUCT_FIELD_Y_NAME, STRUCT_FIELD_Y_TYPE),
            ],
        );

        const MODULE_NAME: &str = "math";
        let module_math = create_module_def(MODULE_NAME, vec![vector_2_struct.into()]);

        /*
         * var mut vec = math::Vector2 {
         *     x: 0.0, y: 2.0
         * }
         */
        const VAR_NAME: &str = "vec";
        const FIELD_X_VAL: f64 = 0.0;
        const FIELD_Y_VAL: f64 = 2.0;
        let var_def_vec = create_var_def(
            VAR_NAME,
            Mutability::Mutable,
            Type::Auto,
            Some(create_struct_lit(
                STRUCT_NAME,
                &[
                    (STRUCT_FIELD_X_NAME, create_decimal_lit(FIELD_X_VAL)),
                    (STRUCT_FIELD_Y_NAME, create_decimal_lit(FIELD_Y_VAL)),
                ],
            )),
        );

        let main_func = create_main_func_def(vec![var_def_vec.into()]);

        /*
         * module math {
         *     struct Vector2 {
         *         x: f32
         *         y: f32
         *     }
         * }
         * func main() {
         *
         *     vec.x = 2.0
         * }";
         */
        let mut program = Hir::from(Block {
            is_global: true,
            statements: vec![module_math.into(), main_func.into()],
            ..Default::default()
        });

        let mut analyzer = Analyzer::new();

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        res.expect("Expected no errors");
    }

    #[test]
    fn incorrect_struct_work_test() {
        // Given
        let mut analyzer = Analyzer::new();

        /* struct MyStruct\
         * {\
         *     f1: i32\
         *     f2: f32\
         * }\
         */
        const STRUCT_NAME: &str = "MyStruct";
        const STRUCT_FIELD_1_NAME: &str = "f1";
        const STRUCT_FIELD_1_TYPE: Type = Type::I32;
        const STRUCT_FIELD_2_NAME: &str = "f2";
        const STRUCT_FIELD_2_TYPE: Type = Type::F32;
        let struct_def = create_struct_def(
            STRUCT_NAME,
            &[
                (STRUCT_FIELD_1_NAME, STRUCT_FIELD_1_TYPE),
                (STRUCT_FIELD_2_NAME, STRUCT_FIELD_2_TYPE),
            ],
        );

        /* var mut s = MyStruct {
         *     f1: 1,
         *     f2: 2.0
         * }
         */
        const VAR_NAME: &str = "s";
        let var_def = create_var_def(
            VAR_NAME,
            Mutability::Mutable,
            Type::Auto,
            Some(create_struct_lit(
                STRUCT_NAME,
                &[
                    (STRUCT_FIELD_1_NAME, create_integer_lit(1)),
                    (STRUCT_FIELD_2_NAME, create_decimal_lit(2.0)),
                ],
            )),
        );

        /* func main() {
         *     <var_def>
         *     s.f1 = 3.0
         *     s.f2 = 2.0
         *     s.f3 = 1.0
         * }";
         */
        let main_func = create_main_func_def(vec![var_def.into()]);

        let mut program = Hir::from(Block {
            is_global: true,
            statements: vec![struct_def.into(), main_func.into()],
            ..Default::default()
        });

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        const EXPECTED_1: &str =
            "Semantic error: Cannot perform operation on objects with different types: i32 and f32";
        const EXPECTED_2: &str = "Semantic error: \"s\" doesn't have member named \"f3\"";

        let messages = res.expect_err("Expected errors");
        let errors = messages.errors_ref();

        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].text, EXPECTED_1);
        assert_eq!(errors[1].text, EXPECTED_2);
    }
    */

    /*
    #[test]
    fn internal_struct_work_test() {
        // Given
        let mut analyzer = Analyzer::new();

        /*
         * struct Unit {
         *     value: f32
         * }
         */
        const STRUCT_1_NAME: &str = "Unit";
        const STRUCT_1_FIELD_1_NAME: &str = "value";
        const STRUCT_1_FIELD_1_TYPE: Type = Type::F32;

        let struct_def_1 = create_struct_def(
            STRUCT_1_NAME,
            &[(STRUCT_1_FIELD_1_NAME, STRUCT_1_FIELD_1_TYPE)],
        );

        /*
         * struct Point2 {
         *     x: Unit
         *     y: Unit
         * }
         */
        const STRUCT_2_NAME: &str = "Point2";
        const STRUCT_2_FIELD_X_NAME: &str = "x";
        const STRUCT_2_FIELD_Y_NAME: &str = "y";
        let struct_2_field_x_type = Type::Custom(Name::from(STRUCT_1_NAME.to_string()));
        let struct_2_field_y_type = Type::Custom(Name::from(STRUCT_1_NAME.to_string()));

        let struct_def_2 = create_struct_def(
            STRUCT_2_NAME,
            &[
                (STRUCT_2_FIELD_X_NAME, struct_2_field_x_type),
                (STRUCT_2_FIELD_Y_NAME, struct_2_field_y_type),
            ],
        );

        /*
         * module math {
         *     <struct_def_1>
         *     <struct_def_2>
         * }
         */
        const MODULE_NAME: &str = "math";
        let module_def =
            create_module_def(MODULE_NAME, vec![struct_def_1.into(), struct_def_2.into()]);

        /*
         * var mut pnt = math::Point2 {
         *     x: math::Unit { value: 1.0 },
         *     y: math::Unit { value: 2.0 },
         * }\
         */
        const VAR_NAME: &str = "pnt";
        let var_def = create_var_def(
            VAR_NAME,
            Mutability::Mutable,
            Type::Auto,
            Some(create_struct_lit(
                STRUCT_2_NAME,
                &[(STRUCT_2_FIELD_X_NAME), (STRUCT_2_FIELD_Y_NAME)],
            )),
        );

        /*
         * func main() {
         *     <var_def>
         *     pnt.x.value = 2.0
         * }";
         */
        const MAIN_FUNC_NAME: &str = "main";
        let main_func_def = create_func_def(MAIN_FUNC_NAME, vec![var_def.into()]);

        /*
         * <module_def>
         * <main_func>
         */
        let mut program = Hir::from(Block {
            is_global: true,
            statements: vec![module_def.into(), main_func_def.into()],
            ..Default::default()
        });

        // When
        program.accept_mut(&mut analyzer).unwrap();

        // Then

        let messages = analyzer.messages_ref();
        if messages.has_errors() {
            panic!("{:?}", messages.errors_ref());
        }
    }
    */

    /*

    #[test]
    fn struct_with_methods_test() {
        const SRC_TEXT: &str = "\nstruct MyStruct\
                                \n{\
                                \n    f1: i32\
                                \n    f2: f32\
                                \n}\
                                \nimpl MyStruct\
                                \n{\
                                \n    func new(): MyStruct {\
                                \n        return MyStruct {\
                                \n                 f1: 0, f2: 0.0\
                                \n               }\
                                \n    }
                                \n}\
                                \nfunc main() {\
                                \n    var s = MyStruct { \
                                \n              f1: 1, f2: 2.0\
                                \n            }\
                                \n}";

        let mut parser = Parser::from_text(SRC_TEXT);

        let ast = parser.parse_program().unwrap();

        let mut hir = {
            let mut lowering = AstLowering::new();
            lowering.low(ast.as_ref()).unwrap()
        };

        {
            let mut analyzer = Analyzer::new();
            analyzer.analyze_program(hir.as_mut()).unwrap();
        }

        {
            const HEADER_EXPECTED: &str = "typedef struct {\
                                         \n    signed int f1;\
                                         \n    float f2;\
                                         \n} MyStruct;\
                                         \nMyStruct MyStruct__new();\
                                         \nvoid main();\n";

            const SOURCE_EXPECTED: &str = "MyStruct MyStruct__new()\
                                         \n{\
                                         \n    return (MyStruct)\
                                         \n    {\
                                         \n        .f1=0,\
                                         \n        .f2=0.0,\
                                         \n    };\
                                         \n}\
                                        \nvoid main()\
                                        \n{\
                                        \n    MyStruct const s = (MyStruct)\
                                        \n    {\
                                        \n        .f1=1,\
                                        \n        .f2=2.0,\
                                        \n    };\
                                        \n}\n";

            let mut header_buffer = Vec::<u8>::new();
            let mut source_buffer = Vec::<u8>::new();
            let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer);

            writer.codegen_program(hir.as_ref()).unwrap();

            let header_res = String::from_utf8(header_buffer).unwrap();
            assert_str_eq!(HEADER_EXPECTED, header_res);

            let source_res = String::from_utf8(source_buffer).unwrap();
            assert_str_eq!(SOURCE_EXPECTED, source_res);
        }
    }

     */
}
