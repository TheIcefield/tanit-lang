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

type Stream = std::fs::File;

pub trait IAst {
    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()>;
}

pub trait GetType {
    fn get_type(&self) -> Option<types::Type> {
        None
    }
}

#[derive(Clone)]
pub enum Ast {
    GScope { node: scopes::Scope },

    LScope { node: scopes::Scope },

    ModuleDef { node: modules::Node },

    StructDef { node: structs::StructNode },

    EnumDef { node: structs::EnumNode },

    FuncDef { node: functions::Node },

    VariableDef { node: variables::Node },

    Value { node: values::Value },

    TypeDecl { node: types::Type },

    AliasDef { node: types::Alias },

    Expression { node: Box<expressions::Expression> },

    BranchStmt { node: branches::Branch },

    BreakStmt { node: branches::Break },

    ContinueStmt { node: branches::Continue },

    ReturnStmt { node: branches::Return },
}

impl Ast {
    pub fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {
        match self {
            Self::GScope { node } => node.traverse(stream, intent),
            Self::LScope { node } => node.traverse(stream, intent),
            Self::ModuleDef { node } => node.traverse(stream, intent),
            Self::StructDef { node } => node.traverse(stream, intent),
            Self::EnumDef { node } => node.traverse(stream, intent),
            Self::FuncDef { node } => node.traverse(stream, intent),
            Self::VariableDef { node } => node.traverse(stream, intent),
            Self::Value { node } => node.traverse(stream, intent),
            Self::TypeDecl { node } => node.traverse(stream, intent),
            Self::AliasDef { node } => node.traverse(stream, intent),
            Self::Expression { node } => node.traverse(stream, intent),
            Self::BranchStmt { node } => node.traverse(stream, intent),
            Self::BreakStmt { node } => node.traverse(stream, intent),
            Self::ContinueStmt { node } => node.traverse(stream, intent),
            Self::ReturnStmt { node } => node.traverse(stream, intent),
        }
    }

    pub fn get_type(&self) -> Option<types::Type> {
        match self {
            Self::Expression { node } => node.get_type(),
            Self::AliasDef { node } => node.get_type(),
            _ => None,
        }
    }
}
