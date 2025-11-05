use tanitc_hir::{
    hir::{
        definitions::functions::{FunctionDef, FunctionParam},
        expressions::Expression,
        types::{FuncType, FuncTypeParam, Type},
    },
    visitor::VisitorMut,
};
use tanitc_lexer::location::Location;
use tanitc_messages::Message;

use crate::{
    symbol_table::entry::{Entry, FuncDefData, SymbolKind},
    AnalyzeResult, Analyzer,
};

impl Analyzer {
    pub(crate) fn analyze_func_def(
        &mut self,
        func_def: &mut FunctionDef,
        is_method: bool,
    ) -> AnalyzeResult<()> {
        if self.has_symbol(func_def.name.id) {
            return Err(Message::multiple_ids(func_def.location, func_def.name.id));
        }

        func_def.name.prefix = self.table.get_id();

        let mut scope_info = self.table.get_scope_info();
        scope_info.safety = func_def.attributes.safety;
        scope_info.is_in_func = true;

        self.table.enter_scope(scope_info);

        self.analyze_func_def_params(func_def, is_method)?;
        let parameters = self.get_func_def_params(func_def)?;

        if let Some(body) = &mut func_def.body {
            self.visit_block(body)?;
        }

        self.table.exit_scope();

        self.analyze_return_type(&mut func_def.return_type, func_def.location)?;

        self.add_symbol(Entry {
            name: func_def.name.id,
            is_static: false,
            kind: SymbolKind::from(FuncDefData {
                ty: FuncType {
                    parameters,
                    return_type: Box::new(func_def.return_type.clone()),
                    safety: func_def.attributes.safety,
                },
                name: func_def.name,
                is_virtual: false,
                is_inline: false,
                no_return: func_def.return_type == Type::unit(),
            }),
        });

        Ok(())
    }

    pub(crate) fn access_func_def(&mut self, rhs: &mut Expression) -> AnalyzeResult<()> {
        let Expression::Call(call_expr) = rhs else {
            return Err(Message::from_string(
                rhs.location(),
                format!("Unexpected rhs: {rhs:?}"),
            ));
        };

        self.analyze_call_expr(call_expr)?;

        Ok(())
    }

    fn analyze_func_def_params(
        &mut self,
        func_def: &mut FunctionDef,
        is_method: bool,
    ) -> AnalyzeResult<()> {
        for (index, param) in func_def.parameters.iter_mut().enumerate() {
            match param {
                FunctionParam::Common(var_def) => {
                    if let Err(err) = self.visit_variable_def(var_def) {
                        self.error(err);
                    }
                }
                FunctionParam::SelfPtr(_)
                | FunctionParam::SelfRef(_)
                | FunctionParam::SelfVal(_) => {
                    if !is_method {
                        self.error(Message::from_string(
                            func_def.location,
                            format!(
                                "In definition of function \"{}\": \"self\" parameter is allowed only in associated functions",
                                func_def.name.id),
                        ));
                    }

                    if index > 0 {
                        self.error(Message::from_string(
                            func_def.location,
                            format!(
                                "In definition of function \"{}\": Unexpected \"self\" parameter. Must be the first parameter of the associated function",
                                func_def.name.id
                            )));
                    }
                }
            }
        }

        Ok(())
    }

    fn get_func_def_params(&self, func_def: &FunctionDef) -> AnalyzeResult<Vec<FuncTypeParam>> {
        let mut parameters = Vec::<FuncTypeParam>::with_capacity(func_def.parameters.len());

        for param in func_def.parameters.iter() {
            if let FunctionParam::Common(var_def) = param {
                parameters.push(FuncTypeParam {
                    id: Some(var_def.identifier),
                    ty: Box::new(var_def.var_type.clone()),
                });
            }
        }

        Ok(parameters)
    }

    fn analyze_return_type(
        &mut self,
        return_type: &mut Type,
        location: Location,
    ) -> AnalyzeResult<()> {
        let Some(type_info) = self.table.lookup_type(return_type) else {
            return Err(Message::undefined_type(location, return_type.to_string()));
        };

        *return_type = type_info.ty;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tanitc_hir::hir::Hir;
    use tanitc_hir_test::{create_block, create_func_def, create_main_func_def};

    /*
    #[test]
    fn function_def_test() {
        const SRC_TEXT: &str = "\nsafe pub func sum(mut a: f32, b: f32): f32 {\
                                \n    return a + b\
                                \n}\
                                \nunsafe func main() {\
                                \n    var ret: f32 = sum(a, b)\
                                \n}";

        let mut parser = Parser::from_text(SRC_TEXT);

        let ast = parser.parse_program().unwrap();

        let hir = {
            let mut lowering = AstLowering::new();
            lowering.low(ast.as_ref()).unwrap()
        };

        {
            const HEADER_EXPECTED: &str = "float sum(float a, float const b);\
                                         \nvoid main();\n";
            const SOURCE_EXPECTED: &str = "float sum(float a, float const b)\
                                         \n{\
                                         \n    return a + b;\
                                         \n}\
                                         \nvoid main()\
                                         \n{\
                                         \n    float const ret = sum(a, b);\
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

    #[test]
    fn functions_test() {
        const SRC_TEXT: &str = "\nfunc f(a: i32, b: i32): f32 {\
                                \n    return a + b\
                                \n}\
                                \n\
                                \nfunc void_func() {\
                                \n}\
                                \n\
                                \nfunc main() {\
                                \n   var param = 34\
                                \n   var res = f(56, b: param)\
                                \n   void_func()
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
            const HEADER_EXPECTED: &str = "float f(signed int const a, signed int const b);\
                                         \nvoid void_func();\
                                         \nvoid main();\n";

            const SOURCE_EXPECTED: &str = "float f(signed int const a, signed int const b)\
                                         \n{\
                                         \n    return a + b;\
                                         \n}\
                                         \nvoid void_func() { }\
                                         \nvoid main()\
                                         \n{\
                                         \n    signed int const param = 34;\
                                         \n    float const res = f(56, param);\
                                         \n    void_func();\
                                         \n}\n";

            let mut header_buffer = Vec::<u8>::new();
            let mut source_buffer = Vec::<u8>::new();
            let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer);

            writer.codegen_program(hir.as_ref()).unwrap();

            let mut res = String::from_utf8(header_buffer).unwrap();
            assert_str_eq!(HEADER_EXPECTED, res);

            res = String::from_utf8(source_buffer).unwrap();
            assert_str_eq!(SOURCE_EXPECTED, res);
        }
    }

    #[test]
    fn function_in_module_work_test() {
        const SRC_TEXT: &str = "\nmodule color {\
                                \n    enum Color {\
                                \n        Red\
                                \n        Green\
                                \n        Blue\
                                \n    }\
                                \n    func get_green(): Color {\
                                \n        var ret = Color::Green\
                                \n        return ret\
                                \n    }\
                                \n}\
                                \nfunc main() {\
                                \n    var green = color::get_green()\
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
            const HEADER_EXPECTED: &str = "typedef enum {\
                                         \n    Red = 0,\
                                         \n    Green = 1,\
                                         \n    Blue = 2,\
                                         \n} color__Color;\
                                         \ncolor__Color color__get_green();\
                                         \nvoid main();\n";

            const SOURCE_EXPECTED: &str = "color__Color color__get_green()\
                                         \n{\
                                         \n    color__Color const ret = 1;\
                                         \n    return ret;\
                                         \n}\
                                         \nvoid main()\
                                         \n{\
                                         \n    void const green = color__get_green();\
                                         \n}\n";

            let mut header_buffer = Vec::<u8>::new();
            let mut source_buffer = Vec::<u8>::new();
            let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer);

            writer.codegen_program(hir.as_ref()).unwrap();

            let mut res = String::from_utf8(header_buffer).unwrap();
            assert_str_eq!(HEADER_EXPECTED, res);

            res = String::from_utf8(source_buffer).unwrap();
            assert_str_eq!(SOURCE_EXPECTED, res);
        }
    }

    #[test]
    fn incorrect_call_test() {
        const SRC_TEXT: &str = "\nfunc f(a: i32, b: i32): f32 {\
                                \n    return a + b\
                                \n}\
                                \n\
                                \nfunc main() {\
                                \n   var pi = 3.14\
                                \n   var res = f(5.6, b: pi)\
                                \n}";

        let mut parser = Parser::from_text(SRC_TEXT);

        let ast = parser.parse_program().unwrap();

        let mut hir = {
            let mut lowering = AstLowering::new();
            lowering.low(ast.as_ref()).unwrap()
        };

        let mut analyzer = Analyzer::new();
        {
            const EXPECTED_1: &str = "Semantic error: Mismatched types. In function \"f\" call: positional parameter \"0\" has type \"f32\" but expected \"i32\"";
            const EXPECTED_2: &str = "Semantic error: Mismatched types. In function \"f\" call: notified parameter \"b\" has type \"f32\" but expected \"i32\"";

            let messages = analyzer.analyze_program(hir.as_mut()).err().unwrap();
            let errors = messages.errors_ref();

            assert_eq!(errors.len(), 2);
            assert_str_eq!(errors[0].text, EXPECTED_1);
            assert_str_eq!(errors[1].text, EXPECTED_2);
        }
    }

    #[test]
    fn incorrect_notified_call_test() {
        const SRC_TEXT: &str = "\nfunc f(a: i32, b: i32): f32 {\
                                \n    return a + b\
                                \n}\
                                \n\
                                \nfunc main() {\
                                \n   var res = f(a: 44, 56)\
                                \n}";

        let mut parser = Parser::from_text(SRC_TEXT);

        let ast = parser.parse_program().unwrap();

        let mut hir = {
            let mut lowering = AstLowering::new();
            lowering.low(ast.as_ref()).unwrap()
        };

        let mut analyzer = Analyzer::new();
        {
            const EXPECTED: &str = "Semantic error: In function \"f\" call: positional parameter \"1\" must be passed before notified";

            let messages = analyzer.analyze_program(hir.as_mut()).err().unwrap();
            let errors = messages.errors_ref();

            assert_eq!(errors.len(), 1);
            assert_str_eq!(errors[0].text, EXPECTED);
        }
    }

    #[test]
    fn incorrect_module_func_call_test() {
        const SRC_TEXT: &str = "\nmodule math {\
                                \n    func f(a: i32, b: i32): f32 {\
                                \n        return a + b\
                                \n    }\
                                \n}\
                                \n\
                                \nfunc main() {\
                                \n   var pi = 3.14\
                                \n   var res = math::f(5.6, b: pi)\
                                \n}";

        let mut parser = Parser::from_text(SRC_TEXT);

        let ast = parser.parse_program().unwrap();

        let mut hir = {
            let mut lowering = AstLowering::new();
            lowering.low(ast.as_ref()).unwrap()
        };

        {
            const EXPECTED_1: &str = "Semantic error: Mismatched types. In function \"f\" call: positional parameter \"0\" has type \"f32\" but expected \"i32\"";
            const EXPECTED_2: &str = "Semantic error: Mismatched types. In function \"f\" call: notified parameter \"b\" has type \"f32\" but expected \"i32\"";

            let mut analyzer = Analyzer::new();

            let messages = analyzer.analyze_program(hir.as_mut()).err().unwrap();
            let errors = messages.errors_ref();

            assert_eq!(errors.len(), 2);
            assert_str_eq!(errors[0].text, EXPECTED_1);
            assert_str_eq!(errors[1].text, EXPECTED_2);
        }
    }
    */

    #[test]
    fn main_not_existing_bad_test() {
        // Given
        const FUNC_1_NAME: &str = "func_1";
        let func_1_def = create_func_def(FUNC_1_NAME, vec![], Type::I32, vec![]);

        const FUNC_2_NAME: &str = "func_2";
        let func_2_def = create_func_def(FUNC_2_NAME, vec![], Type::F64, vec![]);

        let mut program = Hir::from(create_block(vec![func_1_def.into(), func_2_def.into()]));

        let mut analyzer = Analyzer::new();

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        const EXPECTED_ERR: &str = "Semantic error: No entry point!";

        let messages = res.expect_err("Expected errors");
        let errors = messages.errors_ref();

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].text, EXPECTED_ERR);
    }

    #[test]
    fn main_existing_good_test() {
        // Given
        const FUNC_NAME: &str = "func_1";
        let func_def = create_func_def(FUNC_NAME, vec![], Type::F32, vec![]);
        let main_func_def = create_main_func_def(vec![]);

        let mut program = Hir::from(create_block(vec![func_def.into(), main_func_def.into()]));

        let mut analyzer = Analyzer::new();

        // When
        program.accept_mut(&mut analyzer).unwrap();

        let messages = analyzer.messages_ref();
        if messages.has_errors() {
            panic!("{:#?}", messages.errors_ref());
        }
    }

    #[test]
    fn main_bad_type_test() {
        // Given
        const FUNC_NAME: &str = "func_1";
        let func_def = create_func_def(FUNC_NAME, vec![], Type::I32, vec![]);

        const MAIN_FUNC_NAME: &str = "main";
        let main_func_def = create_func_def(MAIN_FUNC_NAME, vec![], Type::F64, vec![]);

        let mut program = Hir::from(create_block(vec![func_def.into(), main_func_def.into()]));

        let mut analyzer = Analyzer::new();

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        const EXPECTED_ERR: &str = "Semantic error: Bad type of main function: f64";

        let messages = res.expect_err("Expected errors");
        let errors = messages.errors_ref();

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].text, EXPECTED_ERR);
    }

    #[test]
    fn main_good_type_i32_test() {
        // Given
        const FUNC_NAME: &str = "func_1";
        let func_def = create_func_def(FUNC_NAME, vec![], Type::I32, vec![]);
        let main_func_def = create_main_func_def(vec![]);

        let mut program = Hir::from(create_block(vec![func_def.into(), main_func_def.into()]));

        let mut analyzer = Analyzer::new();

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        res.expect("Expected no errors");
    }

    #[test]
    fn main_good_type_unit_test() {
        // Given
        const FUNC_NAME: &str = "func_1";
        let func_def = create_func_def(FUNC_NAME, vec![], Type::I32, vec![]);

        const MAIN_FUNC_NAME: &str = "main";
        let main_func_def = create_func_def(MAIN_FUNC_NAME, vec![], Type::unit(), vec![]);

        let mut program = Hir::from(create_block(vec![func_def.into(), main_func_def.into()]));

        let mut analyzer = Analyzer::new();

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        res.expect("Expected no errors");
    }
}
