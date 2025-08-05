use tanitc_ast::ast::variables::VariableDef;
use tanitc_ty::{ArraySize, Type};

use crate::c_generator::CodeGenStream;

use std::io::Write;

impl CodeGenStream<'_> {
    pub fn generate_variable_def(&mut self, var_def: &VariableDef) -> Result<(), std::io::Error> {
        if let Type::Array { .. } = var_def.var_type.get_type() {
            return self.generate_variable_array_def(var_def);
        }

        self.generate_type_spec(&var_def.var_type)?;

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

        Ok(())
    }

    fn generate_variable_array_def(&mut self, var_def: &VariableDef) -> Result<(), std::io::Error> {
        let ty = var_def.var_type.get_type();
        let Type::Array { size, value_type } = ty else {
            unreachable!("Called generate_variable_array_def on none array variable");
        };

        let ArraySize::Fixed(size) = size else {
            unreachable!("Array size must be known at this point");
        };

        let type_str = value_type.get_c_type();
        let var_name = var_def.identifier;
        let mutable_str = if var_def.mutability.is_mutable() {
            " "
        } else {
            " const "
        };

        write!(self, "{type_str}{mutable_str}{var_name}[{size}]")?;

        Ok(())
    }
}
