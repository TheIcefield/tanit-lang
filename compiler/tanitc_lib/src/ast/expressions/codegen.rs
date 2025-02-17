use super::{Expression, ExpressionType};

use tanitc_codegen::{CodeGenMode, CodeGenStream, Codegen};
use tanitc_lexer::token::Lexem;

impl Codegen for Expression {
    fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        use std::io::Write;
        let old_mode = stream.mode;
        stream.mode = CodeGenMode::SourceOnly;

        match &self.expr {
            ExpressionType::Unary { operation, node } => {
                write!(stream, "{}", operation)?;
                node.codegen(stream)?;
            }
            ExpressionType::Binary {
                operation: Lexem::Assign,
                lhs,
                rhs,
            } => {
                lhs.codegen(stream)?;
                write!(stream, " = ")?;
                rhs.codegen(stream)?;
            }
            ExpressionType::Binary {
                operation,
                lhs,
                rhs,
            } => {
                // write!(stream, "(")?;
                lhs.codegen(stream)?;
                write!(stream, " {} ", operation)?;
                rhs.codegen(stream)?;
                // write!(stream, ")")?;
            }
            ExpressionType::Conversion { lhs, ty } => {
                write!(stream, "(({})", ty.get_c_type())?;
                lhs.codegen(stream)?;
                write!(stream, ")")?;
            }
        }

        stream.mode = old_mode;
        Ok(())
    }
}
