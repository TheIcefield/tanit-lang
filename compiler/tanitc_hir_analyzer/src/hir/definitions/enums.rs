use crate::{
    symbol_table::entry::{Entry, EnumData, EnumDefData, EnumDefEntries, SymbolKind},
    AnalyzeResult, Analyzer,
};
use tanitc_hir::hir::definitions::enums::{EnumDef, EnumUnits};
use tanitc_messages::Message;
use tanitc_name::NameSpec;

impl Analyzer {
    pub(crate) fn analyze_enum_def(&mut self, enum_def: &mut EnumDef) -> AnalyzeResult<()> {
        let enum_id = enum_def
            .name
            .get_id()
            .ok_or(Message::empty_name_spec(enum_def.location))?;

        if self.has_symbol(enum_id) {
            return Err(Message::multiple_ids(enum_def.location, enum_id));
        }

        // Copies table.table_path to start of enum_def.name.path
        enum_def.name.path.splice(0..0, self.table.get_path());

        let units = self.analyze_enum_def_units(&enum_def.name, &mut enum_def.units)?;

        self.add_symbol(Entry {
            id: enum_id,
            is_static: true,
            kind: SymbolKind::from(EnumDefData {
                name: enum_def.name.clone(),
                units,
            }),
        });

        Ok(())
    }

    fn analyze_enum_def_units(
        &mut self,
        enum_name: &NameSpec,
        enum_units: &mut EnumUnits,
    ) -> AnalyzeResult<EnumDefEntries> {
        let mut counter = 0usize;
        let mut enums_entries = EnumDefEntries::new();

        for (unit_id, unit_value) in enum_units.iter_mut() {
            if let Some(value) = unit_value {
                counter = *value;
            }

            // mark unmarked enum fields
            *unit_value = Some(counter);

            let unit_data = EnumData {
                name: enum_name.clone(),
                value: counter,
            };
            let entry = Entry {
                id: *unit_id,
                is_static: true,
                kind: unit_data.into(),
            };

            enums_entries.insert(*unit_id, entry);

            counter += 1;
        }

        Ok(enums_entries)
    }
}

#[cfg(test)]
mod tests {
    use tanitc_attributes::Mutability;
    use tanitc_hir::hir::type_spec::Type;
    use tanitc_hir_test::{
        create_enum_def, create_main_func_def, create_module_def, create_program,
        create_scope_resolutions_expr, create_var_def,
    };

    use crate::Analyzer;

    #[test]
    fn good_enum_access_test() {
        // Given
        const ENUM_NAME: &str = "MyEnum";
        const FIRST_UNIT_NAME: &str = "First";
        const SECOND_UNIT_NAME: &str = "Second";
        const MAX_UNIT_NAME: &str = "Max";
        let enum_def = create_enum_def(
            ENUM_NAME,
            vec![
                (FIRST_UNIT_NAME, Some(1)),
                (SECOND_UNIT_NAME, None),
                (MAX_UNIT_NAME, None),
            ],
        );

        let var_value = Some(create_scope_resolutions_expr(&[
            ENUM_NAME,
            SECOND_UNIT_NAME,
        ]));
        let var_def = create_var_def("second", Mutability::Immutable, Type::Auto, var_value);

        let main_func = create_main_func_def(vec![var_def.into()]);

        /* enum MyEnum {
         *     One: 1
         *     Second
         *     Max
         * }
         *
         * func main() {
         *     var a = MyEnum::Second
         * };
         */
        let mut program = create_program(vec![enum_def.into(), main_func.into()]);

        let mut analyzer = Analyzer::new();

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        res.expect("Expected no errors");
    }

    #[test]
    fn good_enum_in_module_access_test() {
        // Given
        const ENUM_NAME: &str = "MyEnum";
        const FIRST_UNIT_NAME: &str = "First";
        const SECOND_UNIT_NAME: &str = "Second";
        const MAX_UNIT_NAME: &str = "Max";
        let enum_def = create_enum_def(
            ENUM_NAME,
            vec![
                (FIRST_UNIT_NAME, Some(1)),
                (SECOND_UNIT_NAME, None),
                (MAX_UNIT_NAME, None),
            ],
        );

        const MODULE_NAME: &str = "MyModule";
        let module_def = create_module_def(MODULE_NAME, vec![enum_def.into()]);

        let var_value = Some(create_scope_resolutions_expr(&[
            MODULE_NAME,
            ENUM_NAME,
            SECOND_UNIT_NAME,
        ]));
        let var_def = create_var_def("second", Mutability::Immutable, Type::Auto, var_value);

        let main_func = create_main_func_def(vec![var_def.into()]);

        /*
         * module MyModule
         *     enum MyEnum {
         *         One: 1
         *         Second
         *         Max
         *     }
         * }
         *
         * func main() {
         *     var a = MyModule::MyEnum::Second
         * };
         */
        let mut program = create_program(vec![module_def.into(), main_func.into()]);

        let mut analyzer = Analyzer::new();

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        res.expect("Expected no errors");
    }

    #[test]
    fn bad_enum_access_test() {
        // Given
        const ENUM_NAME: &str = "MyEnum";
        const FIRST_UNIT_NAME: &str = "First";
        const SECOND_UNIT_NAME: &str = "Second";
        const MAX_UNIT_NAME: &str = "Max";
        let enum_def = create_enum_def(
            ENUM_NAME,
            vec![
                (FIRST_UNIT_NAME, Some(1)),
                (SECOND_UNIT_NAME, None),
                (MAX_UNIT_NAME, None),
            ],
        );

        let var_value = Some(create_scope_resolutions_expr(&[ENUM_NAME, "BadUnit"]));
        let var_def = create_var_def("second", Mutability::Immutable, Type::Auto, var_value);

        let main_func = create_main_func_def(vec![var_def.into()]);

        /* enum MyEnum {
         *     One: 1
         *     Second
         *     Max
         * }
         *
         * func main() {
         *     var a = MyEnum::Second
         * };
         */
        let mut program = create_program(vec![enum_def.into(), main_func.into()]);

        let mut analyzer = Analyzer::new();

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        const EXPECTED_ERR: &str = "Semantic error: enum \"MyEnum\" doesn't contain \"BadUnit\"";

        let messages = res.expect_err("Expected errors");
        let errors = messages.errors_ref();

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].text, EXPECTED_ERR);
    }

    #[test]
    fn bad_enum_in_module_access_test() {
        // Given
        const ENUM_NAME: &str = "MyEnum";
        const FIRST_UNIT_NAME: &str = "First";
        const SECOND_UNIT_NAME: &str = "Second";
        const MAX_UNIT_NAME: &str = "Max";
        let enum_def = create_enum_def(
            ENUM_NAME,
            vec![
                (FIRST_UNIT_NAME, Some(1)),
                (SECOND_UNIT_NAME, None),
                (MAX_UNIT_NAME, None),
            ],
        );

        const MODULE_NAME: &str = "MyModule";
        let module_def = create_module_def(MODULE_NAME, vec![enum_def.into()]);

        let var_value = Some(create_scope_resolutions_expr(&[
            ENUM_NAME,
            SECOND_UNIT_NAME,
        ]));
        let var_def = create_var_def("second", Mutability::Immutable, Type::Auto, var_value);

        let main_func = create_main_func_def(vec![var_def.into()]);

        /*
         * module MyModule
         *     enum MyEnum {
         *         One: 1
         *         Second
         *         Max
         *     }
         * }
         *
         * func main() {
         *     var a = MyEnum::Second
         * };
         */
        let mut program = create_program(vec![module_def.into(), main_func.into()]);

        let mut analyzer = Analyzer::new();

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        const EXPECTED_ERR: &str = "Semantic error: undefined id: \"MyEnum\"";

        let messages = res.expect_err("Expected errors");
        let errors = messages.errors_ref();

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].text, EXPECTED_ERR);
    }

    #[test]
    fn bad_enum_unit_in_module_access_test() {
        // Given
        const ENUM_NAME: &str = "MyEnum";
        const FIRST_UNIT_NAME: &str = "First";
        const SECOND_UNIT_NAME: &str = "Second";
        const MAX_UNIT_NAME: &str = "Max";
        let enum_def = create_enum_def(
            ENUM_NAME,
            vec![
                (FIRST_UNIT_NAME, Some(1)),
                (SECOND_UNIT_NAME, None),
                (MAX_UNIT_NAME, None),
            ],
        );

        const MODULE_NAME: &str = "MyModule";
        let module_def = create_module_def(MODULE_NAME, vec![enum_def.into()]);

        let var_value = Some(create_scope_resolutions_expr(&[
            MODULE_NAME,
            ENUM_NAME,
            "BadUnit",
        ]));
        let var_def = create_var_def("second", Mutability::Immutable, Type::Auto, var_value);

        let main_func = create_main_func_def(vec![var_def.into()]);

        /*
         * module MyModule
         *     enum MyEnum {
         *         One: 1
         *         Second
         *         Max
         *     }
         * }
         *
         * func main() {
         *     var a = MyEnum::Second
         * };
         */
        let mut program = create_program(vec![module_def.into(), main_func.into()]);

        let mut analyzer = Analyzer::new();

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        const EXPECTED_ERR: &str =
            "Semantic error: enum \"MyModule::MyEnum\" doesn't contain \"BadUnit\"";

        let messages = res.expect_err("Expected errors");
        let errors = messages.errors_ref();

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].text, EXPECTED_ERR);
    }
}
