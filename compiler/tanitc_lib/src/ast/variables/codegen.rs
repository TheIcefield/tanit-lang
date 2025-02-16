use super::VariableDef;
use tanitc_codegen::{CodeGenStream, Codegen};

impl Codegen for VariableDef {
    fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        use std::io::Write;

        self.var_type.codegen(stream)?;

        write!(
            stream,
            "{}{}",
            if self.is_mutable { " " } else { " const " },
            self.identifier
        )?;

        Ok(())
    }
}
