use tanitc_hir::hir::definitions::{functions::FunctionDef, methods::ImplDef};
use tanitc_messages::Message;

use crate::Analyzer;

impl<'a> Analyzer<'a> {
    pub(crate) fn analyze_impl_def(&mut self, impl_def: &mut ImplDef) -> Result<(), Message> {
        if self.table.lookup_name_spec(&impl_def.name).is_err() {
            return Err(Message::new(
                impl_def.location,
                format!("Cannot find object named \"{}\"", impl_def.name),
            ));
        };

        self.analyze_impl_methods(&mut impl_def.methods)?;

        Ok(())
    }

    fn analyze_impl_methods(&mut self, methods: &mut [FunctionDef]) -> Result<(), Message> {
        for method in methods.iter_mut() {
            const IS_METHOD: bool = true;

            match self.analyze_func_def(method, IS_METHOD) {
                Ok(_) => {}
                Err(err) => self.error(err),
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tanitc_attributes::Mutability;
    use tanitc_hir::hir::{
        definitions::{functions::FunctionParam, variables::VariableDef},
        type_spec::Type,
    };
    use tanitc_hir_test::{
        create_func_def, create_impl_def, create_main_func_def, create_program, create_struct_def,
    };
    use tanitc_ident::Ident;
    use tanitc_options::CompileOptions;

    fn get_common_param(name: &str) -> FunctionParam {
        FunctionParam::Common(VariableDef {
            identifier: Ident::from(name.to_string()),
            var_type: Type::I32,
            ..Default::default()
        })
    }

    #[test]
    fn self_in_beginning_good_test() {
        // Given
        const STRUCT_NAME: &str = "MyStruct";

        let impl_def_node = create_impl_def(
            STRUCT_NAME,
            vec![create_func_def(
                "by_self",
                vec![
                    FunctionParam::SelfVal(Mutability::Immutable),
                    get_common_param("hello"),
                ],
                Type::unit(),
                vec![],
            )],
        );

        let mut program = create_program(vec![
            create_struct_def(STRUCT_NAME, vec![]).into(),
            impl_def_node.into(),
            create_main_func_def(vec![]).into(),
        ]);

        let compile_options = CompileOptions::default();
        let mut analyzer = Analyzer::new(&compile_options);

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        res.expect("Expected no errors.")
    }

    #[test]
    fn self_in_middle_test() {
        // Given
        const STRUCT_NAME: &str = "MyStruct";

        let impl_def_node = create_impl_def(
            STRUCT_NAME,
            vec![create_func_def(
                "by_self",
                vec![
                    get_common_param("hello"),
                    FunctionParam::SelfVal(Mutability::Immutable),
                ],
                Type::unit(),
                vec![],
            )],
        );

        let mut program = create_program(vec![
            create_struct_def(STRUCT_NAME, vec![]).into(),
            impl_def_node.into(),
            create_main_func_def(vec![]).into(),
        ]);

        let compile_options = CompileOptions::default();
        let mut analyzer = Analyzer::new(&compile_options);

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        const EXPECTED_ERR: &str = "Semantic error: In definition of function \"by_self\": Unexpected \"self\" parameter. Must be the first parameter of the associated function";

        let messages = res.expect_err("Expected errors");
        let errors = messages.errors_ref();

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].text, EXPECTED_ERR);
    }

    #[test]
    fn self_in_func_test() {
        // Given
        let func_def = create_func_def(
            "by_self",
            vec![
                FunctionParam::SelfVal(Mutability::Immutable),
                get_common_param("hello"),
            ],
            Type::unit(),
            vec![],
        );

        let mut program =
            create_program(vec![func_def.into(), create_main_func_def(vec![]).into()]);

        let compile_options = CompileOptions::default();
        let mut analyzer = Analyzer::new(&compile_options);

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        const EXPECTED_ERR: &str = "Semantic error: In definition of function \"by_self\": \"self\" parameter is allowed only in associated functions";
        let messages = res.expect_err("Expected errors");
        let errors = messages.errors_ref();

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].text, EXPECTED_ERR);
    }

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
