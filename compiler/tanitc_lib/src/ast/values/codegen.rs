use super::{CallParam, Value, ValueType};
use tanitc_codegen::{CodeGenStream, Codegen};

use std::io::Write;

impl Codegen for CallParam {
    fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        match self {
            Self::Positional(_, node) => node.codegen(stream),
            Self::Notified(..) => unreachable!("Notified CallParam is not allowed in codegen"),
        }
    }
}

impl Codegen for Value {
    fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        match &self.value {
            ValueType::Integer(val) => write!(stream, "{}", *val)?,
            ValueType::Decimal(val) => write!(stream, "{}", *val)?,
            ValueType::Identifier(val) => val.codegen(stream)?,
            ValueType::Call {
                identifier,
                arguments,
            } => {
                /* at this point, all arguments must be converted to positional */

                identifier.codegen(stream)?;
                write!(stream, "(")?;

                if !arguments.is_empty() {
                    arguments[0].codegen(stream)?;
                }

                for arg in arguments.iter().skip(1) {
                    write!(stream, ", ")?;
                    arg.codegen(stream)?;
                }

                write!(stream, ")")?;
            }
            _ => unimplemented!(),
        }

        Ok(())
    }
}
