use super::Scope;
use crate::ast::Ast;
use crate::codegen::{CodeGenStream, Codegen};

use std::io::Write;

impl Codegen for Scope {
    fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        if !self.is_global {
            writeln!(stream, "{{")?;
        }

        for stmt in self.statements.iter() {
            stmt.codegen(stream)?;

            match stmt {
                Ast::Expression { .. }
                | Ast::BreakStmt { .. }
                | Ast::ContinueStmt { .. }
                | Ast::ReturnStmt { .. }
                | Ast::VariableDef { .. } => write!(stream, ";")?,
                _ => {}
            }

            writeln!(stream)?;
        }

        if !self.is_global {
            writeln!(stream, "}}")?;
        }
        Ok(())
    }
}
