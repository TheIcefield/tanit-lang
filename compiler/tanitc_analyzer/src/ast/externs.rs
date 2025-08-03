use tanitc_ast::{ast::externs::ExternDef, visitor::VisitorMut};
use tanitc_messages::Message;

use crate::Analyzer;

impl Analyzer {
    pub fn analyze_extern_def(&mut self, extern_def: &mut ExternDef) -> Result<(), Message> {
        for func_def in extern_def.functions.iter_mut() {
            if let Err(err) = self.visit_func_def(func_def) {
                self.error(err);
            }
        }

        Ok(())
    }
}
