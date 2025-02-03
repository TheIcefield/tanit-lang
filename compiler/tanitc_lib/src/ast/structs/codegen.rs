use super::StructDef;
use tanitc_codegen::{CodeGenMode, CodeGenStream, Codegen};

impl Codegen for StructDef {
    fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        use std::io::Write;

        let old_mode = stream.mode;
        stream.mode = CodeGenMode::HeaderOnly;

        writeln!(stream, "typedef struct {{")?;
        for (field_id, field_type) in self.fields.iter() {
            field_type.codegen(stream)?;
            write!(stream, " ")?;
            field_id.codegen(stream)?;
            writeln!(stream, ";")?;
        }
        write!(stream, "}} ")?;

        self.identifier.codegen(stream)?;

        writeln!(stream, ";")?;

        stream.mode = old_mode;
        Ok(())
    }
}
