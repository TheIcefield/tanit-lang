use tanitc_hir::hir::definitions::modules::{ModuleDef, ModuleDefBody};

use crate::CodeGenStream;

impl CodeGenStream<'_> {
    pub fn generate_module_def(&mut self, module_def: &ModuleDef) -> std::io::Result<()> {
        match &module_def.body {
            ModuleDefBody::External(body) => self.generate(body),
            ModuleDefBody::Internal(body) => self.generate_block(body),
        }
    }
}
