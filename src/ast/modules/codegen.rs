use super::ModuleDef;
use crate::codegen::{CodeGenStream, Codegen};

impl Codegen for ModuleDef {
    fn codegen(&self, _stream: &mut CodeGenStream) -> std::io::Result<()> {
        unimplemented!()
    }
}
