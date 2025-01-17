use crate::analyzer::{Analyze, Analyzer};
use crate::codegen::{CodeGenStream, Codegen};
use crate::messages::Message;
use crate::serializer::{Serialize, XmlWriter};

pub mod aliases;
pub mod branches;
pub mod enums;
pub mod expressions;
pub mod functions;
pub mod identifiers;
pub mod modules;
pub mod scopes;
pub mod structs;
pub mod types;
pub mod values;
pub mod variables;
pub mod variants;
// pub mod externs;

#[derive(Clone, PartialEq)]
pub enum Ast {
    Scope { node: scopes::Scope },

    ModuleDef { node: modules::ModuleDef },

    StructDef { node: structs::StructDef },

    EnumDef { node: enums::EnumDef },

    VariantDef { node: variants::VariantDef },

    FuncDef { node: functions::FunctionDef },

    VariableDef { node: variables::VariableDef },

    Value { node: values::Value },

    Type { node: types::Type },

    AliasDef { node: aliases::AliasDef },

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
            Self::VariantDef { node } => node.serialize(writer),
            Self::EnumDef { node } => node.serialize(writer),
            Self::FuncDef { node } => node.serialize(writer),
            Self::VariableDef { node } => node.serialize(writer),
            Self::Value { node } => node.serialize(writer),
            Self::Type { node } => node.serialize(writer),
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
            Self::Scope { node } => node.analyze(analyzer),
            Self::FuncDef { node } => node.analyze(analyzer),
            Self::AliasDef { node } => node.analyze(analyzer),
            Self::ModuleDef { node } => node.analyze(analyzer),
            Self::StructDef { node } => node.analyze(analyzer),
            Self::EnumDef { node } => node.analyze(analyzer),
            Self::VariantDef { node } => node.analyze(analyzer),
            Self::VariableDef { node } => node.analyze(analyzer),
            Self::Value { node } => node.analyze(analyzer),
            Self::Expression { node } => node.analyze(analyzer),
            Self::BranchStmt { node } => node.analyze(analyzer),
            Self::ContinueStmt { node } => node.analyze(analyzer),
            Self::ReturnStmt { node } => node.analyze(analyzer),
            Self::BreakStmt { node } => node.analyze(analyzer),
            Self::Type { node } => node.analyze(analyzer),
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
            Self::VariantDef { node } => node.codegen(stream),
            Self::FuncDef { node } => node.codegen(stream),
            Self::VariableDef { node } => node.codegen(stream),
            Self::Value { node } => node.codegen(stream),
            Self::Type { node } => node.codegen(stream),
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
