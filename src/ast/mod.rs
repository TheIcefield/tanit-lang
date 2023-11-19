pub mod types;
pub mod values;
pub mod variables;
pub mod functions;
pub mod calls;
pub mod structs;
pub mod branches;
pub mod scopes;
pub mod modules;
pub mod expressions;
// pub mod externs;

use crate::parser::put_intent;
use std::io::Write;

type Stream = std::fs::File;

trait IAst {
    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()>;
}

#[derive(Clone)]
pub enum Ast {
    GScope { node: scopes::Scope, },
    
    LScope { node: scopes::Scope, },
    
    ModuleDef { node: modules::Node, },
    
    StructDef { node: structs::Node },

    FuncDef { node: functions::Node },

    VariableDef { node: variables::Node },

    Value { node: values::ValueType },

    TypeDecl { node: types::Type },

    AliasDef { node: types::Alias },

    Expression { node: Box<expressions::Expression> },

    LoopStmt { node: branches::LoopNode,},

    BreakStmt { node: branches::Break },

    ContinueStmt { node: branches::Continue },

    ReturnStmt { node: branches::Return },
}

impl Ast {
    pub fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {
        match self {
            Ast::GScope { node } => node.traverse(stream, intent)?,
            Ast::LScope { node } => node.traverse(stream, intent)?,
            Ast::ModuleDef { node } => node.traverse(stream, intent)?,
            Ast::StructDef { node } => node.traverse(stream, intent)?,
            Ast::FuncDef { node } => node.traverse(stream, intent)?,
            Ast::VariableDef { node } => {
                writeln!(stream, "{}<definition>", put_intent(intent))?;
                node.traverse(stream, intent + 1)?;
                writeln!(stream, "{}</definition>", put_intent(intent))?;
            },
            Ast::Value { node } => node.traverse(stream, intent)?,
            Ast::TypeDecl { node } => node.traverse(stream, intent)?,
            Ast::AliasDef { node } => node.traverse(stream, intent)?,
            Ast::Expression { node } => node.traverse(stream, intent)?,
            Ast::LoopStmt { node } => node.traverse(stream, intent)?,
            Ast::BreakStmt { node } => node.traverse(stream, intent)?,
            Ast::ContinueStmt { node } => node.traverse(stream, intent)?,
            Ast::ReturnStmt { node } => node.traverse(stream, intent)?,    
        }

        Ok(())
    }
}

