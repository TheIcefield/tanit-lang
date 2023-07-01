pub mod types;
pub mod values;
pub mod variables;
pub mod aliases;
pub mod functions;
// pub mod calls;
pub mod structs;
pub mod branches;
pub mod scopes;
pub mod modules;
pub mod expressions;
// pub mod externs;

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

    VarDef { node: variables::Node },

    TypeDecl { node: types::Node },

    AliasDef { node: aliases::Node },

    Expression { node: Box<expressions::Expression> },

    LoopStmt { node: branches::LoopNode,},

    Value { node: values::ValueType },

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
            Ast::VarDef { node } => node.traverse(stream, intent)?,
            Ast::TypeDecl { node } => node.traverse(stream, intent)?,
            Ast::AliasDef { node } => node.traverse(stream, intent)?,
            Ast::Expression { node } => node.traverse(stream, intent)?,
            Ast::LoopStmt { node } => node.traverse(stream, intent)?,
            Ast::Value { node } => node.traverse(stream, intent)?,
            Ast::BreakStmt { node } => node.traverse(stream, intent)?,
            Ast::ContinueStmt { node } => node.traverse(stream, intent)?,
            Ast::ReturnStmt { node } => node.traverse(stream, intent)?,    
        }

        Ok(())
    }
}

