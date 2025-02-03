use super::{VariantDef, VariantField};
use tanitc_codegen::{CodeGenMode, CodeGenStream, Codegen};

impl Codegen for VariantField {
    fn codegen(&self, _stream: &mut CodeGenStream) -> std::io::Result<()> {
        unimplemented!()
    }
}

impl Codegen for VariantDef {
    fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        use std::io::Write;

        let old_mode = stream.mode;
        stream.mode = CodeGenMode::HeaderOnly;

        writeln!(stream, "typedef enum {{")?;
        for (field_id, _) in self.fields.iter() {
            field_id.codegen(stream)?;
            writeln!(stream, ",")?;
        }
        write!(stream, "}} ")?;

        self.identifier.codegen(stream)?;

        writeln!(stream, ";")?;

        stream.mode = old_mode;
        Ok(())
    }
}
