use tanitc_ast::ast::modules::ModuleDef;

use crate::c_generator::{CodeGenMode, CodeGenStream};

use std::io::Write;

impl CodeGenStream<'_> {
    pub fn generate_module_def(&mut self, module_def: &ModuleDef) -> Result<(), std::io::Error> {
        if module_def.body.is_none() {
            self.generate_external_module(module_def)?;
        } else {
            self.generate_internal_module(module_def)?;
        }

        Ok(())
    }

    fn generate_internal_module(&mut self, module_def: &ModuleDef) -> std::io::Result<()> {
        if let Some(body) = &module_def.body {
            self.generate_block(body)?;
        }

        Ok(())
    }

    fn generate_external_module(&mut self, module_def: &ModuleDef) -> std::io::Result<()> {
        let old_mode = self.mode;
        self.mode = CodeGenMode::HeaderOnly;

        writeln!(self, "#include \"./{}.tt.h\"", module_def.identifier)?;

        self.mode = old_mode;

        Ok(())
    }
}
