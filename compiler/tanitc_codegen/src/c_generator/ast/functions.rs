use tanitc_ast::ast::functions::{FunctionDef, FunctionParam};
use tanitc_ident::Ident;

use crate::c_generator::{CodeGenMode, CodeGenStream};

use std::io::{ErrorKind, Write};

impl CodeGenStream<'_> {
    pub fn generate_func_def(
        &mut self,
        func_def: &FunctionDef,
        struct_name: Option<Ident>,
    ) -> Result<(), std::io::Error> {
        let old_mode = self.mode;
        self.mode = if func_def.body.is_some() {
            CodeGenMode::Both
        } else {
            CodeGenMode::HeaderOnly
        };

        let indentation = self.indentation();

        write!(self, "{indentation}")?;

        self.generate_type_spec(&func_def.return_type)?;

        let full_name = if let Some(struct_name) = struct_name {
            format!("{struct_name}__{}", func_def.identifier)
        } else {
            format!("{}", func_def.identifier)
        };

        write!(self, " {full_name}")?;

        self.generate_func_def_params(func_def, struct_name)?;

        self.mode = CodeGenMode::HeaderOnly;
        writeln!(self, ";")?;

        if let Some(body) = &func_def.body {
            self.mode = CodeGenMode::SourceOnly;
            if body.statements.is_empty() {
                writeln!(self, " {{ }}")?;
            } else {
                writeln!(self)?;
                self.generate_block(body)?;
            }
        }

        self.mode = old_mode;
        Ok(())
    }

    fn generate_func_def_param(
        &mut self,
        param: &FunctionParam,
        struct_name: Option<Ident>,
    ) -> Result<(), std::io::Error> {
        match param {
            FunctionParam::SelfVal(mutability) => {
                let Some(struct_name) = struct_name else {
                    return Err(std::io::Error::from(ErrorKind::InvalidData));
                };

                write!(
                    self,
                    "{struct_name} {}self",
                    if mutability.is_const() { "const " } else { "" }
                )
            }
            FunctionParam::SelfRef(mutability) | FunctionParam::SelfPtr(mutability) => {
                let Some(struct_name) = struct_name else {
                    return Err(std::io::Error::from(ErrorKind::InvalidData));
                };

                write!(
                    self,
                    "{struct_name} {}* const self",
                    if mutability.is_const() { "const " } else { "" }
                )
            }
            FunctionParam::Common(var_def) => self.generate_variable_def(var_def),
        }
    }

    fn generate_func_def_params(
        &mut self,
        func_def: &FunctionDef,
        struct_name: Option<Ident>,
    ) -> Result<(), std::io::Error> {
        write!(self, "(")?;
        if !func_def.parameters.is_empty() {
            let param = func_def.parameters.first().unwrap();
            self.generate_func_def_param(param, struct_name)?;
        }

        for param in func_def.parameters.iter().skip(1) {
            write!(self, ", ")?;
            self.generate_func_def_param(param, struct_name)?;
        }
        write!(self, ")")?;

        Ok(())
    }
}
