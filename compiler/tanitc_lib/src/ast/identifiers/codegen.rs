use super::{Identifier, IdentifierType};

use tanitc_codegen::{CodeGenStream, Codegen};

use std::io::Write;

impl Codegen for Identifier {
    fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        if let IdentifierType::Common(id) = &self.identifier {
            write!(stream, "{}", id)?;
        }

        if let IdentifierType::Complex(ids) = &self.identifier {
            write!(stream, "{}", ids[0])?;
            for id in ids.iter().skip(1) {
                write!(stream, "__{}", id)?;
            }
        }

        Ok(())
    }
}
