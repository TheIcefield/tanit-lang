use tanitc_ast::ast::modules::ModuleDef;

use crate::c_generator::CodeGenStream;

impl CodeGenStream<'_> {
    pub fn generate_module_def(&mut self, module_def: &ModuleDef) -> Result<(), std::io::Error> {
        self.generate_block(module_def.body.as_ref())?;

        Ok(())
    }
}
