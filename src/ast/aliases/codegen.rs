use super::AliasDef;
use crate::codegen::{CodeGenMode, CodeGenStream, Codegen};

impl Codegen for AliasDef {
    fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        use std::io::Write;

        let old_mode = stream.mode;
        stream.mode = CodeGenMode::HeaderOnly;

        write!(stream, "typedef {} ", self.value.get_c_type())?;

        self.identifier.codegen(stream)?;

        stream.mode = old_mode;
        writeln!(stream, ";\n")
    }
}
