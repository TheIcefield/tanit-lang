use super::{Branch, BranchType, Interupter, InterupterType};

use tanitc_codegen::{CodeGenMode, CodeGenStream, Codegen};

use std::io::Write;

impl Codegen for Branch {
    fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        let old_mode = stream.mode;
        stream.mode = CodeGenMode::SourceOnly;
        match &self.branch {
            BranchType::Loop { body } => {
                write!(stream, "while (1)")?;

                body.codegen(stream)?;
            }
            BranchType::While { body, condition } => {
                write!(stream, "while (")?;

                condition.codegen(stream)?;

                writeln!(stream, ")")?;

                body.codegen(stream)?;
            }
            BranchType::If { condition, body } => {
                write!(stream, "if (")?;
                condition.codegen(stream)?;
                writeln!(stream, ")")?;

                body.codegen(stream)?;
            }
            BranchType::Else { body } => {
                writeln!(stream, "else")?;
                body.codegen(stream)?;
            }
        }
        stream.mode = old_mode;
        Ok(())
    }
}

impl Codegen for Interupter {
    fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        let old_mode = stream.mode;
        stream.mode = CodeGenMode::SourceOnly;

        write!(stream, "return ")?;

        if let InterupterType::Return { ret } = &self.interupter {
            if let Some(expr) = ret.as_ref() {
                expr.codegen(stream)?;
            }
        }

        stream.mode = old_mode;
        Ok(())
    }
}
