use super::FunctionDef;
use crate::codegen::{CodeGenMode, CodeGenStream, Codegen};
use std::io::Write;

impl Codegen for FunctionDef {
    fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        let old_mode = stream.mode;
        stream.mode = if self.body.is_some() {
            CodeGenMode::Both
        } else {
            CodeGenMode::HeaderOnly
        };

        self.return_type.codegen(stream)?;

        write!(stream, " ")?;

        self.identifier.codegen(stream)?;

        // generate parameters
        write!(stream, "(")?;
        if !self.parameters.is_empty() {
            self.parameters[0].codegen(stream)?;
        }

        for param in self.parameters.iter().skip(1) {
            write!(stream, ", ")?;
            param.codegen(stream)?;
        }
        write!(stream, ")")?;

        stream.mode = CodeGenMode::HeaderOnly;
        writeln!(stream, ";")?;

        if let Some(body) = &self.body {
            stream.mode = CodeGenMode::SourceOnly;
            body.codegen(stream)?;
        }

        stream.mode = old_mode;
        Ok(())
    }
}
