use tanitc_ast::ast::types::TypeSpec;

use crate::c_generator::CodeGenStream;

impl CodeGenStream<'_> {
    pub fn generate_type_spec(&mut self, type_spec: &TypeSpec) -> Result<(), std::io::Error> {
        use std::io::Write;

        write!(self, "{}", type_spec.get_c_type())
    }
}
