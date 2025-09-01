use tanitc_ast::ast::expressions::{BinaryOperation, Expression, ExpressionKind, UnaryOperation};

use crate::c_generator::{CodeGenMode, CodeGenStream};

impl CodeGenStream<'_> {
    pub fn generate_expression(&mut self, expr: &Expression) -> Result<(), std::io::Error> {
        use std::io::Write;

        let old_mode = self.mode;
        self.mode = CodeGenMode::SourceOnly;

        match &expr.kind {
            ExpressionKind::Unary { operation, node } => {
                match operation {
                    UnaryOperation::RefMut | UnaryOperation::Ref => write!(self, "&")?,
                    UnaryOperation::Not => write!(self, "~")?,
                    UnaryOperation::Deref => write!(self, "*")?,
                };

                self.generate(node)?;
            }
            ExpressionKind::Binary {
                operation: BinaryOperation::Assign,
                lhs,
                rhs,
            } => {
                self.generate(lhs)?;
                write!(self, " = ")?;
                self.generate(rhs)?;
            }
            ExpressionKind::Binary {
                operation,
                lhs,
                rhs,
            } => {
                // write!(self, "(")?;
                self.generate(lhs)?;
                write!(self, " {operation} ")?;
                self.generate(rhs)?;
                // write!(self, ")")?;
            }
            ExpressionKind::Conversion { lhs, ty } => {
                write!(self, "(({})", ty.get_c_type())?;
                self.generate(lhs)?;
                write!(self, ")")?;
            }
            ExpressionKind::Access { lhs, rhs } => {
                self.generate(lhs)?;
                write!(self, "__")?;
                self.generate(rhs)?;
            }
            ExpressionKind::Get { lhs, rhs } => {
                self.generate(lhs)?;
                write!(self, ".")?;
                self.generate(rhs)?;
            }
            ExpressionKind::Indexing { lhs, index } => {
                self.generate(lhs)?;

                write!(self, "[")?;
                self.generate(index)?;
                write!(self, "]")?;
            }
            ExpressionKind::Term { node, .. } => {
                self.generate(node)?;
            }
        }

        self.mode = old_mode;
        Ok(())
    }
}
