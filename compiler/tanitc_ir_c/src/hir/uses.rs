use tanitc_hir::hir::uses::Use;

use crate::CodeGenStream;

impl CodeGenStream<'_> {
    pub fn generate_use(&mut self, _u: &Use) -> Result<(), std::io::Error> {
        Ok(())
    }
}
