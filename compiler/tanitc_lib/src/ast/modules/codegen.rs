use super::ModuleDef;
use crate::codegen::{CodeGenMode, CodeGenStream, Codegen};

impl Codegen for ModuleDef {
    fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        if self.body.is_none() {
            self.generate_external_module(stream)
        } else {
            self.generate_internal_module(stream)
        }
    }
}

impl ModuleDef {
    fn generate_internal_module(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        if let Some(body) = &self.body {
            body.codegen(stream)?;
        }

        Ok(())
    }

    fn generate_external_module(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        use std::io::Write;

        let old_mode = stream.mode;
        stream.mode = CodeGenMode::HeaderOnly;

        writeln!(stream, "#include \"./{}.tt.h\"", self.identifier)?;

        stream.mode = old_mode;

        Ok(())
    }
}
