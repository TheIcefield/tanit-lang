pub mod branches;
pub mod calls;
pub mod expressions;
pub mod functions;
pub mod modules;
pub mod scopes;
pub mod structs;
pub mod types;
pub mod values;
pub mod variables;
// pub mod externs;

use crate::parser::put_intent;
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
}
