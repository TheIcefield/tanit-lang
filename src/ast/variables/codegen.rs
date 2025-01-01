use super::VariableDef;
use crate::codegen::{CodeGenStream, Codegen};

impl Codegen for VariableDef {
    fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        use std::io::Write;

        self.var_type.codegen(stream)?;

        write!(stream, "{}", if self.is_mutable { " " } else { " const " })?;

        self.identifier.codegen(stream)?;

        Ok(())
    }
}
