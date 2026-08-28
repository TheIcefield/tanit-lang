use tanitc_hir::hir::{definitions::variables::VariableDef, type_spec::Type};

use crate::CodeGenStream;

use std::io::Write;

impl CodeGenStream<'_> {
    pub fn generate_variable_def(&mut self, var_def: &VariableDef) -> std::io::Result<()> {
        if matches!(var_def.var_type, Type::Array { .. }) {
            return self.generate_variable_array_def(var_def);
        }

        self.generate_type(&var_def.var_type)?;

        write!(
            self,
            "{}{}",
            if var_def.mutability.is_mutable() {
                " "
            } else {
                " const "
            },
            var_def.identifier
        )?;

        if let Some(value) = &var_def.value {
            write!(self, " = ")?;
            self.generate_expression(value)?;
        }

        Ok(())
    }

    fn generate_variable_array_def(&mut self, var_def: &VariableDef) -> std::io::Result<()> {
        let ty = &var_def.var_type;
        let Type::Array(array_type) = ty else {
            unreachable!("Called generate_variable_array_def on none array variable");
        };

        let type_str = array_type.internal_type.get_c_type();
        let var_name = var_def.identifier;
        let mutable_str = if var_def.mutability.is_mutable() {
            " "
        } else {
            " const "
        };

        write!(self, "{type_str}{mutable_str}{var_name}[")?;

        self.generate_expression(&array_type.size)?;

        write!(self, "]")?;

        Ok(())
    }
}
