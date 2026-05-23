use tanitc_hir::hir::{definitions::aliases::AliasDef, type_spec::Type};
use tanitc_messages::Message;

use crate::{
    symbol_table::entry::{AliasDefData, Entry, SymbolKind},
    AnalyzeResult, Analyzer,
};

impl Analyzer {
    pub(crate) fn analyze_alias_def(&mut self, alias_def: &mut AliasDef) -> AnalyzeResult<()> {
        let alias_id = alias_def
            .name
            .get_id()
            .ok_or(Message::empty_name_spec(alias_def.location))?;

        if self.has_symbol(alias_id) {
            return Err(Message::multiple_ids(alias_def.location, alias_id));
        }

        // Copies table.table_path to start of alias_def.name.path
        alias_def.name.path.splice(0..0, self.table.get_path());

        self.add_symbol(Entry {
            id: alias_id,
            is_static: true,
            kind: AliasDefData {
                ty: alias_def.value.get_type(),
            }
            .into(),
        });

        Ok(())
    }

    pub(crate) fn find_alias_value(&self, alias_type: &Type) -> Option<Type> {
        let Type::Custom(type_name) = alias_type else {
            return None;
        };

        let entry = self.table.lookup_name_spec(type_name).ok()?;

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
    use super::*;

    use tanitc_attributes::Mutability;
    use tanitc_hir_test::{
        create_alias_def, create_custom_type, create_decimal_lit, create_integer_lit,
        create_main_func_def, create_program, create_struct_def, create_struct_lit, create_var_def,
    };

    #[test]
    fn struct_with_alias_typed_fields_test() {
        // Given
        const FIRST_ALIAS: &str = "VecUnit";
        let common_alias_def = create_alias_def(FIRST_ALIAS, Type::F32);

        const STRUCT_NAME: &str = "Vec2";
        const FIELD_1_NAME: &str = "x";
        const FIELD_2_NAME: &str = "y";

        let struct_def = create_struct_def(
            STRUCT_NAME,
            vec![
                (FIELD_1_NAME, create_custom_type(&[FIRST_ALIAS])),
                (FIELD_2_NAME, create_custom_type(&[FIRST_ALIAS])),
            ],
        );

        const SECOND_ALIAS: &str = "Vec";
        let alias_to_struct = create_alias_def(SECOND_ALIAS, create_custom_type(&[STRUCT_NAME]));

        let var_value = Some(create_struct_lit(
            &[SECOND_ALIAS],
            vec![
                (FIELD_1_NAME, create_decimal_lit(10.0)),
                (FIELD_2_NAME, create_decimal_lit(10.0)),
            ],
        ));

        let var_def = create_var_def("v", Mutability::Immutable, Type::Auto, var_value);

        let main_func = create_main_func_def(vec![var_def.into()]);

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
        let mut program = create_program(vec![
            common_alias_def.into(),
            struct_def.into(),
            alias_to_struct.into(),
            main_func.into(),
        ]);

        let mut analyzer = Analyzer::new();

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        res.expect("Expected no errors");
    }

    #[test]
    fn bad_alias_object_test() {
        // Given
        const ALIAS_NAME: &str = "Vec";
        let alias_def = create_alias_def(ALIAS_NAME, Type::I32);

        let var_value = Some(create_struct_lit(
            &[ALIAS_NAME],
            vec![
                ("x", create_decimal_lit(10.0)),
                ("y", create_decimal_lit(10.0)),
            ],
        ));
        let var_def = create_var_def("v", Mutability::Immutable, Type::Auto, var_value);

        let main_func = create_main_func_def(vec![var_def.into()]);

        /*
         * alias Vec = i32
         * func main() {
         *     var v = Vec { x: 10.0, y: 10.0 }
         * };
         */
        let mut program = create_program(vec![alias_def.into(), main_func.into()]);

        let mut analyzer = Analyzer::new();

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        const EXPECTED_ERR: &str = "Semantic error: Common type \"i32\" does not have any fields";
        let messages = res.expect_err("Expected errors");
        let errors = messages.errors_ref();

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].text, EXPECTED_ERR);
    }

    #[test]
    fn alias_common_type_test() {
        // Given
        const ALIAS_NAME: &str = "A";
        let alias_def = create_alias_def(ALIAS_NAME, Type::I32);

        let var_value = Some(create_integer_lit(100));

        let var_def = create_var_def(
            "a",
            Mutability::Immutable,
            create_custom_type(&[ALIAS_NAME]),
            var_value,
        );

        let main_func = create_main_func_def(vec![var_def.into()]);

        /*
         * alias A = i32
         * func main() {
         *     var a: A = 100
         * }
         */
        let mut program = create_program(vec![alias_def.into(), main_func.into()]);

        let mut analyzer = Analyzer::new();

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        res.expect("Expected no errors");
    }

    #[test]
    fn bad_alias_common_type_test() {
        // Given
        const ALIAS_NAME: &str = "A";
        let alias_def = create_alias_def(ALIAS_NAME, Type::I32);

        let var_value = Some(create_decimal_lit(std::f64::consts::PI));
        let var_def = create_var_def(
            "a",
            Mutability::Immutable,
            create_custom_type(&[ALIAS_NAME]),
            var_value,
        );

        let main_func = create_main_func_def(vec![var_def.into()]);

        /*
         * alias A = i32
         * func main() {
         *     var a: A = 3.14
         * }
         */
        let mut program = create_program(vec![alias_def.into(), main_func.into()]);

        let mut analyzer = Analyzer::new();

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        const EXPECTED_ERR: &str = "Semantic error: Cannot perform operation on objects with different types: A (aka: i32) and f32";

        let messages = res.expect_err("Expected errors");
        let errors = messages.errors_ref();

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].text, EXPECTED_ERR);
    }

    #[test]
    fn alias_custom_type_test() {
        // Given
        const STRUCT_NAME: &str = "S";
        let struct_def = create_struct_def(STRUCT_NAME, vec![]);

        const ALIAS_NAME: &str = "A";
        let alias_def = create_alias_def(ALIAS_NAME, create_custom_type(&[STRUCT_NAME]));

        let var_value = Some(create_struct_lit(&[STRUCT_NAME], vec![]));
        let var_def = create_var_def(
            "a",
            Mutability::Immutable,
            create_custom_type(&[ALIAS_NAME]),
            var_value,
        );

        let main_func = create_main_func_def(vec![var_def.into()]);

        /*
         * struct S {}
         * alias A = S
         * func main() {
         *     var a: A = S {}
         * }";
         */
        let mut program =
            create_program(vec![struct_def.into(), alias_def.into(), main_func.into()]);

        let mut analyzer = Analyzer::new();

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        res.expect("Expected no errors");
    }

    #[test]
    fn bad_alias_custom_type_test() {
        // Given
        const STRUCT_NAME: &str = "S";
        let struct_def = create_struct_def(STRUCT_NAME, vec![]);

        const ALIAS_NAME: &str = "A";
        let alias_def = create_alias_def(ALIAS_NAME, create_custom_type(&[STRUCT_NAME]));

        let var_value = Some(create_integer_lit(100));
        let var_def = create_var_def(
            "a",
            Mutability::Immutable,
            create_custom_type(&[ALIAS_NAME]),
            var_value,
        );

        let main_func_def = create_main_func_def(vec![var_def.into()]);

        /*
         * struct S {}
         * alias A = S
         * func main() {
         *     var a: A = 100
         * }";
         */
        let mut program = create_program(vec![
            struct_def.into(),
            alias_def.into(),
            main_func_def.into(),
        ]);

        let mut analyzer = Analyzer::new();

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        const EXPECTED_ERR: &str = "Semantic error: Cannot perform operation on objects with different types: A (aka: S) and i32";

        let messages = res.expect_err("Expected errors");
        let errors = messages.errors_ref();

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].text, EXPECTED_ERR);
    }

    #[test]
    fn alias_to_alias_type_test() {
        // Given
        const STRUCT_NAME: &str = "S";
        let struct_def = create_struct_def(STRUCT_NAME, vec![]);

        const ALIAS_TO_STRUCT_NAME: &str = "A";
        let alias_to_struct =
            create_alias_def(ALIAS_TO_STRUCT_NAME, create_custom_type(&[STRUCT_NAME]));

        const ALIAS_TO_ALIAS_NAME: &str = "B";
        let alias_to_alias = create_alias_def(
            ALIAS_TO_ALIAS_NAME,
            create_custom_type(&[ALIAS_TO_STRUCT_NAME]),
        );

        let var_value = Some(create_struct_lit(&[STRUCT_NAME], vec![]));
        let var_def = create_var_def(
            "b",
            Mutability::Immutable,
            create_custom_type(&[ALIAS_TO_ALIAS_NAME]),
            var_value,
        );

        let main_func = create_main_func_def(vec![var_def.into()]);

        /*
         * struct S {}
         * alias A = S
         * alias B = A
         *
         * func main() {
         *     var b: B = S {}
         * }
         */
        let mut program = create_program(vec![
            struct_def.into(),
            alias_to_struct.into(),
            alias_to_alias.into(),
            main_func.into(),
        ]);

        let mut analyzer = Analyzer::new();

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        res.expect("Expected no errors");
    }

    #[test]
    fn bad_alias_to_alias_type_test() {
        // Given
        const STRUCT_NAME: &str = "S";
        let struct_def = create_struct_def(STRUCT_NAME, vec![]);

        const ALIAS_1_NAME: &str = "A";
        let alias_1_def = create_alias_def(ALIAS_1_NAME, create_custom_type(&[STRUCT_NAME]));

        const ALIAS_2_NAME: &str = "B";
        let alias_2_def = create_alias_def(ALIAS_2_NAME, create_custom_type(&[ALIAS_1_NAME]));

        let var_value = Some(create_integer_lit(50));
        let var_def = create_var_def(
            "b",
            Mutability::Immutable,
            create_custom_type(&[ALIAS_2_NAME]),
            var_value,
        );

        let main_func = create_main_func_def(vec![var_def.into()]);

        /*
         * struct S {}
         * alias A = S
         * alias B = A
         * func main() {
         *     var b: B = 50
         * }
         */
        let mut program = create_program(vec![
            struct_def.into(),
            alias_1_def.into(),
            alias_2_def.into(),
            main_func.into(),
        ]);

        let mut analyzer = Analyzer::new();

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        const EXPECTED_ERR: &str = "Semantic error: Cannot perform operation on objects with different types: B (aka: S) and i32";
        let messages = res.expect_err("Expected errors");
        let errors = messages.errors_ref();

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].text, EXPECTED_ERR);
    }
}
