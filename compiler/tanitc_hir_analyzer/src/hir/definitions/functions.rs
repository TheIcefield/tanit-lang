use tanitc_hir::hir::{
    definitions::functions::{FunctionDef, FunctionParam},
    type_spec::{FuncType, FuncTypeParam, Type},
};
use tanitc_ident::Ident;
use tanitc_lexer::location::Location;
use tanitc_messages::Message;

use crate::{
    symbol_table::entry::{Entry, FuncDefData},
    AnalyzeResult, Analyzer,
};

impl Analyzer {
    pub(crate) fn analyze_func_def(
        &mut self,
        func_def: &mut FunctionDef,
        is_method: bool,
    ) -> AnalyzeResult<()> {
        let func_id = func_def
            .name
            .get_id()
            .ok_or(Message::empty_name_spec(func_def.location))?;

        if self.has_symbol(func_id) {
            return Err(Message::multiple_ids(func_def.location, func_id));
        }

        // Copies table.table_path to start of enum_def.name.path
        func_def.name.path.splice(0..0, self.table.get_path());

        let mut scope_info = self.table.get_scope_info();
        scope_info.safety = func_def.attributes.safety;
        scope_info.is_in_func = true;

        self.table.enter_scope(scope_info);

        let parameters = self.analyze_func_def_params(
            func_id,
            &mut func_def.parameters,
            func_def.location,
            is_method,
        )?;

        if let Some(body) = &mut func_def.body {
            self.analyze_block(body)?;
        }

        self.table.exit_scope();

        self.analyze_return_type(&mut func_def.return_type, func_def.location)?;

        let func_def_data = FuncDefData {
            ty: FuncType {
                parameters,
                return_type: Box::new(func_def.return_type.clone()),
                safety: func_def.attributes.safety,
            },
            name: func_def.name.clone(),
            is_virtual: false,
            is_inline: false,
            no_return: func_def.return_type == Type::unit(),
        };

        self.add_symbol(Entry {
            id: func_id,
            is_static: false,
            kind: func_def_data.into(),
        });

        Ok(())
    }

    fn analyze_func_def_params(
        &mut self,
        func_id: Ident,
        func_params: &mut [FunctionParam],
        location: Location,
        is_method: bool,
    ) -> AnalyzeResult<Vec<FuncTypeParam>> {
        let mut parameters = Vec::<FuncTypeParam>::with_capacity(func_params.len());

        for (index, param) in func_params.iter_mut().enumerate() {
            match param {
                FunctionParam::Common(var_def) => {
                    if let Err(err) = self.analyze_variable_def(var_def) {
                        self.error(err);
                        continue;
                    }

                    parameters.push(FuncTypeParam {
                        id: Some(var_def.identifier),
                        ty: Box::new(var_def.var_type.clone()),
                    });
                }
                FunctionParam::SelfPtr(_)
                | FunctionParam::SelfRef(_)
                | FunctionParam::SelfVal(_) => {
                    if !is_method {
                        self.error(Message::new(
                            location,
                            format!(
                                "In definition of function \"{func_id}\": \"self\" parameter is allowed only in associated functions")
                        ));
                    }

                    if index > 0 {
                        self.error(Message::new(
                            location,
                            format!(
                                "In definition of function \"{func_id}\": Unexpected \"self\" parameter. Must be the first parameter of the associated function"
                            )));
                    }
                }
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
    use tanitc_attributes::{Mutability, Safety};
    use tanitc_hir::hir::Hir;
    use tanitc_hir_test::{
        create_block, create_call_expr, create_func_def, create_integer_lit, create_main_func_def,
        create_module_def, create_program, create_var_def,
    };

    #[test]
    fn good_func_access_test() {
        // Given
        const FUNC_NAME: &str = "foo";
        let func_def = create_func_def(FUNC_NAME, vec![], Type::unit(), vec![]);

        let call_expr = create_call_expr(&[FUNC_NAME], vec![]);
        let main_func = create_main_func_def(vec![call_expr.into()]);

        /* func foo() { }
         * func main() {
         *     foo()
         * }
         */
        let mut program = create_program(vec![func_def.into(), main_func.into()]);

        let mut analyzer = Analyzer::new();

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        res.expect("Expected no errors");
    }

    #[test]
    fn good_func_in_module_access_test() {
        // Given
        const FUNC_NAME: &str = "foo";
        let func_def = create_func_def(FUNC_NAME, vec![], Type::unit(), vec![]);

        const MODULE_NAME: &str = "MyModule";
        let module_def = create_module_def(MODULE_NAME, vec![func_def.into()]);

        let call_expr = create_call_expr(&[MODULE_NAME, FUNC_NAME], vec![]);
        let main_func = create_main_func_def(vec![call_expr.into()]);

        /* module MyModule {
         *     func foo() { }
         * }
         * func main() {
         *     MyModule::foo()
         * }
         */
        let mut program = create_program(vec![module_def.into(), main_func.into()]);

        let mut analyzer = Analyzer::new();

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        res.expect("Expected no errors");
    }

    #[test]
    fn good_unsafe_func_in_module_access_test() {
        // Given
        const FUNC_NAME: &str = "foo";
        let mut func_def = create_func_def(FUNC_NAME, vec![], Type::unit(), vec![]);
        func_def.attributes.safety = Safety::Unsafe;

        const MODULE_NAME: &str = "MyModule";
        let module_def = create_module_def(MODULE_NAME, vec![func_def.into()]);

        let call_expr = create_call_expr(&[MODULE_NAME, FUNC_NAME], vec![]);
        let mut main_func = create_main_func_def(vec![call_expr.into()]);
        main_func.attributes.safety = Safety::Unsafe;

        /* module MyModule {
         *     unsafe func foo() { }
         * }
         * unsafe func main() {
         *     MyModule::foo()
         * }
         */
        let mut program = create_program(vec![module_def.into(), main_func.into()]);

        let mut analyzer = Analyzer::new();

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        res.expect("Expected no errors");
    }

    #[test]
    fn good_func_return_type_from_module_call_test() {
        // Given
        const FUNC_NAME: &str = "foo";
        let func_def = create_func_def(FUNC_NAME, vec![], Type::I32, vec![]);

        const MODULE_NAME: &str = "MyModule";
        let module_def = create_module_def(MODULE_NAME, vec![func_def.into()]);

        let call_expr = create_call_expr(&[MODULE_NAME, FUNC_NAME], vec![]);
        let var_def = create_var_def("var", Mutability::Immutable, Type::I32, Some(call_expr));

        let main_func = create_main_func_def(vec![var_def.into()]);

        /* module MyModule {
         *     func foo() -> i32 { }
         * }
         * func main() {
         *     func var_name: i32 = MyModule::foo()
         * }
         */
        let mut program = create_program(vec![module_def.into(), main_func.into()]);

        let mut analyzer = Analyzer::new();

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        res.expect("Expected no errors");
    }

    #[test]
    fn bad_func_access_test() {
        // Given
        const FUNC_NAME: &str = "foo";

        let call_expr = create_call_expr(&[FUNC_NAME], vec![]);
        let main_func = create_main_func_def(vec![call_expr.into()]);

        /* func main() {
         *     foo() # foo is undefined
         * }
         */
        let mut program = create_program(vec![main_func.into()]);

        let mut analyzer = Analyzer::new();

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        const EXPECTED_ERR: &str = "Semantic error: undefined id: \"foo\"";

        let messages = res.expect_err("Expected errors");
        let errors = messages.errors_ref();

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].text, EXPECTED_ERR);
    }

    #[test]
    fn bad_func_in_module_access_test() {
        // Given
        const FUNC_NAME: &str = "foo";
        let func_def = create_func_def(FUNC_NAME, vec![], Type::unit(), vec![]);

        const MODULE_NAME: &str = "MyModule";
        let module_def = create_module_def(MODULE_NAME, vec![func_def.into()]);

        let call_expr = create_call_expr(&[FUNC_NAME], vec![]);
        let main_func = create_main_func_def(vec![call_expr.into()]);

        /* module MyModule {
         *     func foo() { }
         * }
         * func main() {
         *     MyModule::foo()
         * }
         */
        let mut program = create_program(vec![module_def.into(), main_func.into()]);

        let mut analyzer = Analyzer::new();

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        const EXPECTED_ERR: &str = "Semantic error: undefined id: \"foo\"";

        let messages = res.expect_err("Expected errors");
        let errors = messages.errors_ref();

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].text, EXPECTED_ERR);
    }

    #[test]
    fn bad_func_name_in_module_access_test() {
        // Given
        const FUNC_NAME: &str = "foo";
        let func_def = create_func_def(FUNC_NAME, vec![], Type::unit(), vec![]);

        const MODULE_NAME: &str = "MyModule";
        let module_def = create_module_def(MODULE_NAME, vec![func_def.into()]);

        let call_expr = create_call_expr(&[MODULE_NAME, "bar"], vec![]);
        let main_func = create_main_func_def(vec![call_expr.into()]);

        /* module MyModule {
         *     func foo() { }
         * }
         * func main() {
         *     MyModule::bar()
         * }
         */
        let mut program = create_program(vec![module_def.into(), main_func.into()]);

        let mut analyzer = Analyzer::new();

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        const EXPECTED_ERR: &str = "Semantic error: module \"MyModule\" doesn't contain \"bar\"";

        let messages = res.expect_err("Expected errors");
        let errors = messages.errors_ref();

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].text, EXPECTED_ERR);
    }

    #[test]
    fn bad_unsafe_func_in_module_access_test() {
        // Given
        const FUNC_NAME: &str = "foo";
        let mut func_def = create_func_def(FUNC_NAME, vec![], Type::unit(), vec![]);
        func_def.attributes.safety = Safety::Unsafe;

        const MODULE_NAME: &str = "MyModule";
        let module_def = create_module_def(MODULE_NAME, vec![func_def.into()]);

        let call_expr = create_call_expr(&[MODULE_NAME, FUNC_NAME], vec![]);
        let main_func = create_main_func_def(vec![call_expr.into()]);

        /* module MyModule {
         *     unsafe func foo() { }
         * }
         * unsafe func main() {
         *     MyModule::foo()
         * }
         */
        let mut program = create_program(vec![module_def.into(), main_func.into()]);

        let mut analyzer = Analyzer::new();

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        const EXPECTED_ERR: &str =
            "Semantic error: Call unsafe function requires an unsafe function or block";

        let messages = res.expect_err("Expected errors");
        let errors = messages.errors_ref();

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].text, EXPECTED_ERR);
    }

    #[test]
    fn bad_func_param_from_module_call_test() {
        // Given
        const FUNC_NAME: &str = "foo";
        let func_def = create_func_def(FUNC_NAME, vec![], Type::unit(), vec![]);

        const MODULE_NAME: &str = "MyModule";
        let module_def = create_module_def(MODULE_NAME, vec![func_def.into()]);

        let call_expr = create_call_expr(&[MODULE_NAME, FUNC_NAME], vec![create_integer_lit(45)]);
        let main_func = create_main_func_def(vec![call_expr.into()]);

        /* module MyModule {
         *     func foo() { }
         * }
         * func main() {
         *     MyModule::foo(45)
         * }
         */
        let mut program = create_program(vec![module_def.into(), main_func.into()]);

        let mut analyzer = Analyzer::new();

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        const EXPECTED_ERR: &str =
            "Semantic error: Too many arguments passed in function, expected: 0, actually: 1";

        let messages = res.expect_err("Expected errors");
        let errors = messages.errors_ref();

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].text, EXPECTED_ERR);
    }

    #[test]
    fn bad_func_return_type_from_module_call_test() {
        // Given
        const FUNC_NAME: &str = "foo";
        let func_def = create_func_def(FUNC_NAME, vec![], Type::F32, vec![]);

        const MODULE_NAME: &str = "MyModule";
        let module_def = create_module_def(MODULE_NAME, vec![func_def.into()]);

        let call_expr = create_call_expr(&[MODULE_NAME, FUNC_NAME], vec![]);
        let var_def = create_var_def(
            "var_name",
            Mutability::Immutable,
            Type::U16,
            Some(call_expr),
        );

        let main_func = create_main_func_def(vec![var_def.into()]);

        /* module MyModule {
         *     func foo() -> f32 { }
         * }
         * func main() {
         *     var var_name: u16 = MyModule::foo()
         * }
         */
        let mut program = create_program(vec![module_def.into(), main_func.into()]);

        let mut analyzer = Analyzer::new();

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        const EXPECTED_ERR: &str =
            "Semantic error: Cannot perform operation on objects with different types: u16 and f32";

        let messages = res.expect_err("Expected errors");
        let errors = messages.errors_ref();

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].text, EXPECTED_ERR);
    }

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
