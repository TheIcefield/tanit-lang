pub mod branches;
pub mod expressions;
pub mod functions;
pub mod modules;
pub mod scopes;
pub mod structs;
pub mod types;
pub mod values;
pub mod variables;
// pub mod externs;

use crate::{lexer::TokenType, parser::put_intent};
use std::io::Write;

type Stream = std::fs::File;

trait IAst {
    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()>;
}

#[derive(Clone)]
pub enum Ast {
    GScope { node: scopes::Scope },

    LScope { node: scopes::Scope },

    ModuleDef { node: modules::Node },

    StructDef { node: structs::Node },

    FuncDef { node: functions::Node },

    VariableDef { node: variables::Node },

    Value { node: values::Value },

    TypeDecl { node: types::Type },

    AliasDef { node: types::Alias },

    Expression { node: Box<expressions::Expression> },

    IfStmt { node: branches::Branch },

    LoopStmt { node: branches::LoopNode },

    BreakStmt { node: branches::Break },

    ContinueStmt { node: branches::Continue },

    ReturnStmt { node: branches::Return },
}

impl Ast {
    pub fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {
        match self {
            Self::GScope { node } => node.traverse(stream, intent)?,
            Self::LScope { node } => node.traverse(stream, intent)?,
            Self::ModuleDef { node } => node.traverse(stream, intent)?,
            Self::StructDef { node } => node.traverse(stream, intent)?,
            Self::FuncDef { node } => node.traverse(stream, intent)?,
            Self::VariableDef { node } => {
                writeln!(stream, "{}<definition>", put_intent(intent))?;
                node.traverse(stream, intent + 1)?;
                writeln!(stream, "{}</definition>", put_intent(intent))?;
            }
            Self::Value { node } => node.traverse(stream, intent)?,
            Self::TypeDecl { node } => node.traverse(stream, intent)?,
            Self::AliasDef { node } => node.traverse(stream, intent)?,
            Self::Expression { node } => node.traverse(stream, intent)?,
            Self::IfStmt { node } => node.traverse(stream, intent)?,
            Self::LoopStmt { node } => node.traverse(stream, intent)?,
            Self::BreakStmt { node } => node.traverse(stream, intent)?,
            Self::ContinueStmt { node } => node.traverse(stream, intent)?,
            Self::ReturnStmt { node } => node.traverse(stream, intent)?,
        }

        Ok(())
    }

    pub fn get_type(&self) -> Option<types::Type> {
        match self {
            Self::Expression { node } => match node.as_ref() {
                expressions::Expression::Binary {
                    operation,
                    lhs,
                    rhs,
                } => match operation {
                    TokenType::Neq
                    | TokenType::Eq
                    | TokenType::Lt
                    | TokenType::Lte
                    | TokenType::Gt
                    | TokenType::Gte => Some(types::Type::Bool),

                    _ => {
                        let lhs_type = lhs.get_type();
                        let rhs_type = rhs.get_type();

                        if lhs_type == rhs_type {
                            return lhs_type;
                        }

                        None
                    }
                },
                expressions::Expression::Unary { node, .. } => node.get_type(),
            },
            Self::AliasDef { node } => Some(node.value.clone()),
            _ => None,
        }
    }
}
