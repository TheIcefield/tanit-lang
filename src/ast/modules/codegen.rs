use super::ModuleDef;
use crate::codegen::{CodeGenMode, CodeGenStream, Codegen};

impl Codegen for ModuleDef {
    fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        self.generate_external_module(stream)
    }
}

impl ModuleDef {
    fn generate_external_module(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        use std::io::Write;

        let header_name = format!("{}_generated.h", self.identifier);
        let source_name = format!("{}_generated.c", self.identifier);

        let mut header_stream = std::fs::File::create(header_name.clone())
            .expect("Error: can't create header for external AST");
        let mut source_stream = std::fs::File::create(source_name.clone())
            .expect("Error: can't create source for external AST");

        let mut internal_stream = CodeGenStream::new(&mut header_stream, &mut source_stream)
            .expect("Error: can't create codegen for external AST");

        let old_mode = stream.mode;
        stream.mode = CodeGenMode::HeaderOnly;

        writeln!(stream, "#include \"./{}\"", header_name)?;

        stream.mode = old_mode;

        self.body.codegen(&mut internal_stream)
    }
}
