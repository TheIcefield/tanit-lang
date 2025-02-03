use super::EnumDef;

use tanitc_codegen::{CodeGenMode, CodeGenStream, Codegen};

impl Codegen for EnumDef {
    fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        use std::io::Write;

        let old_mode = stream.mode;
        stream.mode = CodeGenMode::HeaderOnly;

        writeln!(stream, "typedef enum {{")?;

        for field in self.fields.iter() {
            writeln!(stream, "    {} = {},", field.0, field.1.unwrap_or_default())?;
        }

        writeln!(stream, "}} {};", self.identifier)?;

        stream.mode = old_mode;

        Ok(())
    }
}
