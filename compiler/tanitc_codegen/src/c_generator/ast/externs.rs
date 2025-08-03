use tanitc_ast::ast::externs::ExternDef;

use crate::c_generator::{CodeGenMode, CodeGenStream};

impl CodeGenStream<'_> {
    pub fn generate_extern_def(&mut self, extern_def: &ExternDef) -> Result<(), std::io::Error> {
        let mode = self.mode;
        self.mode = CodeGenMode::HeaderOnly;

        for func_def in extern_def.functions.iter() {
            self.generate_func_def(func_def, None)?;
        }

        self.mode = mode;

        Ok(())
    }
}
