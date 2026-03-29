use tanitc_hir::hir::{definitions::variables::VariableDef, type_spec::Type};
use tanitc_lexer::location::Location;
use tanitc_messages::Message;

use crate::{
    symbol_table::entry::{Entry, VarDefData, VarStorageType},
    AnalyzeResult, Analyzer,
};

impl Analyzer {
    pub(crate) fn analyze_variable_def(&mut self, var_def: &mut VariableDef) -> AnalyzeResult<()> {
        if self.has_symbol(var_def.identifier) {
            return Err(Message::multiple_ids(var_def.location, var_def.identifier));
        }

        if Type::Auto == var_def.var_type && var_def.value.is_none() {
            return Err(Message::new(
                var_def.location,
                format!(
                    "Type annotation needed for variable named \"{}\"",
                    var_def.identifier
                ),
            ));
        }

        let rhs_type_info = if let Some(rhs) = &mut var_def.value {
            self.analyze_expression(rhs)?;
            Some(self.get_expr_type(rhs))
        } else {
            None
        };

        if let Some(rhs_type) = rhs_type_info {
            if Type::Auto == var_def.var_type {
                // Use rhs type
                var_def.var_type = rhs_type.ty.clone();
            } else {
                // Analyze specified type
                self.analyze_variable_type(&mut var_def.var_type, var_def.location)?;
                self.compare_types(&var_def.var_type, &rhs_type.ty, var_def.location)?;
            }
        }

        let var_def_data = VarDefData {
            storage: VarStorageType::Auto,
            var_type: var_def.var_type.clone(),
            mutability: var_def.mutability,
            is_initialization: true,
        };
        let entry = Entry {
            id: var_def.identifier,
            is_static: false,
            kind: var_def_data.into(),
        };

        self.add_symbol(entry);

        Ok(())
    }

    fn analyze_variable_type(&self, var_type: &mut Type, location: Location) -> AnalyzeResult<()> {
        let Some(type_info) = self.table.lookup_type(var_type) else {
            return Err(Message::undefined_type(location, var_type.to_string()));
        };

        *var_type = type_info.ty;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tanitc_hir::hir::{blocks::Block, Hir};
    use tanitc_hir_test::create_main_func_def;
    use tanitc_ident::Ident;

    #[test]
    fn var_without_type_and_rhs_bad_test() {
        const VAR_NAME: &str = "var";

        const EXPECTED_ERR: &str =
            "Semantic error: Type annotation needed for variable named \"var\"";

        let var = VariableDef {
            identifier: Ident::from(VAR_NAME.to_string()),
            var_type: Type::Auto,
            value: None,
            ..Default::default()
        };

        let main_func_def = create_main_func_def(vec![var.into()]);

        let mut program = Hir::from(Block {
            is_global: true,
            statements: vec![main_func_def.into()],
            ..Default::default()
        });

        let mut analyzer = Analyzer::new();
        program.accept_mut(&mut analyzer).unwrap();

        let errors = analyzer.messages_ref().errors_ref();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].text, EXPECTED_ERR);
    }
}
