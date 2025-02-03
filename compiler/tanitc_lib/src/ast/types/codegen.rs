use super::Type;
use crate::codegen::{CodeGenStream, Codegen};

impl Codegen for Type {
    fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        use std::io::Write;

        write!(stream, "{}", self.get_c_type())
    }
}
