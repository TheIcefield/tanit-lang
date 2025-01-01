use super::{Branch, BranchType, Break, Continue, Return};
use crate::codegen::{CodeGenMode, CodeGenStream, Codegen};

use std::io::Write;

impl Codegen for Branch {
    fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        let old_mode = stream.mode;
        stream.mode = CodeGenMode::SourceOnly;
        match &self.branch {
            BranchType::IfElse {
                condition,
                main_body,
                else_body,
            } => {
                write!(stream, "if (")?;
                condition.codegen(stream)?;
                writeln!(stream, ")")?;

                main_body.codegen(stream)?;

                if let Some(else_body) = else_body {
                    writeln!(stream, "else")?;
                    else_body.codegen(stream)?;
                }
            }
            BranchType::Loop { body, condition } => {
                write!(stream, "while (")?;

                if let Some(condition) = condition {
                    condition.codegen(stream)?;
                } else {
                    write!(stream, "1")?;
                }
                writeln!(stream, ")")?;

                body.codegen(stream)?;
            }
        }
        stream.mode = old_mode;
        Ok(())
    }
}

impl Codegen for Break {
    fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        let old_mode = stream.mode;
        stream.mode = CodeGenMode::SourceOnly;

        write!(stream, "break")?;

        stream.mode = old_mode;
        Ok(())
    }
}

impl Codegen for Continue {
    fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        let old_mode = stream.mode;
        stream.mode = CodeGenMode::SourceOnly;

        write!(stream, "continue")?;

        stream.mode = old_mode;
        Ok(())
    }
}

impl Codegen for Return {
    fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        let old_mode = stream.mode;
        stream.mode = CodeGenMode::SourceOnly;

        write!(stream, "return ")?;
        if let Some(expr) = self.expr.as_ref() {
            expr.codegen(stream)?;
        }

        stream.mode = old_mode;
        Ok(())
    }
}
