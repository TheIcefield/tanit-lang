use tanitc_ast::ast::uses::Use;

use crate::c_generator::CodeGenStream;

impl CodeGenStream<'_> {
    pub fn generate_use(&mut self, _u: &Use) -> Result<(), std::io::Error> {
        Ok(())
    }
}
