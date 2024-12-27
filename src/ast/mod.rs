use crate::analyzer::Analyzer;
use crate::codegen::CodeGenStream;
use crate::messages::Message;
use crate::serializer::XmlWriter;

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

pub trait IAst {
    fn serialize(&self, writer: &mut XmlWriter) -> std::io::Result<()>;

    fn analyze(&mut self, analyzer: &mut Analyzer) -> Result<(), Message>;

    fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()>;

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
    pub fn serialize(&self, writer: &mut XmlWriter) -> std::io::Result<()> {
        match self {
            Self::Scope { node } => node.serialize(writer),
            Self::ModuleDef { node } => node.serialize(writer),
            Self::StructDef { node } => node.serialize(writer),
            Self::EnumDef { node } => node.serialize(writer),
            Self::FuncDef { node } => node.serialize(writer),
            Self::VariableDef { node } => node.serialize(writer),
            Self::Value { node } => node.serialize(writer),
            Self::TypeDecl { node } => node.serialize(writer),
            Self::AliasDef { node } => node.serialize(writer),
            Self::Expression { node } => node.serialize(writer),
            Self::BranchStmt { node } => node.serialize(writer),
            Self::BreakStmt { node } => node.serialize(writer),
            Self::ContinueStmt { node } => node.serialize(writer),
            Self::ReturnStmt { node } => node.serialize(writer),
        }
    }

    pub fn analyze(&mut self, analyzer: &mut Analyzer) -> Result<(), Message> {
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
        }?;

        // TODO: fix conversion
        // if matches!(self, Ast::Expression { .. }) {
        //     expressions::Expression::convert_ast_node(self, analyzer)?;
        // }

        Ok(())
    }

    pub fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        match self {
            Self::Scope { node } => node.codegen(stream),
            Self::ModuleDef { node } => node.codegen(stream),
            Self::StructDef { node } => node.codegen(stream),
            Self::EnumDef { node } => node.codegen(stream),
            Self::FuncDef { node } => node.codegen(stream),
            Self::VariableDef { node } => node.codegen(stream),
            Self::Value { node } => node.codegen(stream),
            Self::TypeDecl { node } => node.codegen(stream),
            Self::AliasDef { node } => node.codegen(stream),
            Self::Expression { node } => node.codegen(stream),
            Self::BranchStmt { node } => node.codegen(stream),
            Self::BreakStmt { node } => node.codegen(stream),
            Self::ContinueStmt { node } => node.codegen(stream),
            Self::ReturnStmt { node } => node.codegen(stream),
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
