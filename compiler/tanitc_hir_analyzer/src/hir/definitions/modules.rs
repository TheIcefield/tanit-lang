use tanitc_hir::hir::definitions::modules::{ModuleDef, ModuleDefBody};
use tanitc_ident::Ident;
use tanitc_messages::Message;

use crate::{
    symbol_table::{
        entry::{Entry, ModuleDefData, SymbolKind},
        table::Table,
    },
    AnalyzeResult, Analyzer,
};

impl Analyzer {
    pub(crate) fn analyze_module_def(&mut self, module_def: &mut ModuleDef) -> AnalyzeResult<()> {
        let module_id = module_def
            .name
            .get_id()
            .ok_or(Message::empty_name_spec(module_def.location))?;

        if self.has_symbol(module_id) {
            return Err(Message::multiple_ids(module_def.location, module_id));
        }

        // Copies table.table_path to start of struct_def.name.path
        module_def.name.path.splice(0..0, self.table.get_path());

        self.table.insert(Entry {
            id: module_id,
            is_static: true,
            kind: SymbolKind::from(ModuleDefData {
                name: module_def.name.clone(),
                table: Box::new(Table::new()),
            }),
        });

        self.analyze_module_def_body(module_id, &mut module_def.body)?;

        Ok(())
    }

    fn analyze_module_def_body(
        &mut self,
        module_id: Ident,
        body: &mut ModuleDefBody,
    ) -> AnalyzeResult<()> {
        let joined_path = self.table.get_joined_path(module_id);

        let mut analyzer = Analyzer::new();
        analyzer.set_compile_options(self.compile_options.clone());
        analyzer.table.set_path(joined_path);

        match body {
            ModuleDefBody::External(body) => body.accept_mut(&mut analyzer)?,
            ModuleDefBody::Internal(body) => analyzer.analyze_block(body)?,
        }

        let entry = self.table.lookup_mut(module_id).unwrap();
        let SymbolKind::ModuleDef(ref mut data) = &mut entry.kind else {
            unreachable!();
        };

        data.table = analyzer.table;

        Ok(())
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

    use super::*;

    #[test]
    fn bad_module_item_access_test() {
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
            "BadItem",
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
        const EXPECTED_ERR: &str =
            "Semantic error: module \"MyModule\" doesn't contain \"BadItem\"";

        let messages = res.expect_err("Expected errors");
        let errors = messages.errors_ref();

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].text, EXPECTED_ERR);
    }
}
