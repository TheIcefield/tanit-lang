use crate::analyzer::Analyzer;

pub mod branches;
pub mod expressions;
pub mod functions;
pub mod identifiers;
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

    fn analyze(&mut self, analyzer: &mut Analyzer) -> Result<(), &'static str>;

    fn get_type(&self, _analyzer: &mut Analyzer) -> types::Type {
        types::Type::Tuple {
            components: Vec::new(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Ast {
    Scope { node: scopes::Scope },

    ModuleDef { node: modules::ModuleNode },

    StructDef { node: structs::StructNode },

    EnumDef { node: structs::EnumNode },

    FuncDef { node: functions::FunctionNode },

    VariableDef { node: variables::VariableNode },

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
            Self::Scope { node } => node.traverse(stream, intent),
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

    pub fn analyze(&mut self, analyzer: &mut Analyzer) -> Result<(), &'static str> {
        let _ = expressions::Expression::convert_ast_node(self, analyzer);
        match self {
            Ast::Scope { node } => node.analyze(analyzer),

            Ast::FuncDef { node } => node.analyze(analyzer),

            Ast::AliasDef { node } => node.analyze(analyzer),

            Ast::ModuleDef { node } => node.analyze(analyzer),

            Ast::StructDef { node } => node.analyze(analyzer),

            Ast::EnumDef { node } => node.analyze(analyzer),

            Ast::VariableDef { node } => node.analyze(analyzer),

            Ast::Value { node } => node.analyze(analyzer),

            Ast::Expression { node } => node.analyze(analyzer),

            Ast::BranchStmt { node } => node.analyze(analyzer),

            Ast::ContinueStmt { node } => node.analyze(analyzer),

            Ast::ReturnStmt { node } => node.analyze(analyzer),

            Ast::BreakStmt { node } => node.analyze(analyzer),

            Ast::TypeDecl { node } => node.analyze(analyzer),
        }
    }

    pub fn get_type(&self, analyzer: &mut Analyzer) -> types::Type {
        match self {
            Self::Expression { node } => node.get_type(analyzer),
            Self::AliasDef { node } => node.get_type(analyzer),
            Self::Value { node } => node.get_type(analyzer),
            Self::VariableDef { node } => node.get_type(analyzer),
            _ => todo!("GetType"),
        }
    }
}
