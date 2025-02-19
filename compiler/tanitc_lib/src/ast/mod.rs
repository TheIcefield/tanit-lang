use tanitc_analyzer::{symbol_table::SymbolTable, Analyze, Analyzer};
use tanitc_codegen::{CodeGenStream, Codegen};
use tanitc_messages::Message;
use tanitc_parser::Parser;
use tanitc_serializer::{Serialize, XmlWriter};
use tanitc_ty::Type;

pub mod aliases;
pub mod branches;
pub mod enums;
pub mod expressions;
pub mod functions;
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
    Scope(scopes::Scope),
    ModuleDef(modules::ModuleDef),
    StructDef(structs::StructDef),
    EnumDef(enums::EnumDef),
    VariantDef(variants::VariantDef),
    FuncDef(functions::FunctionDef),
    VariableDef(variables::VariableDef),
    AliasDef(aliases::AliasDef),
    Expression(expressions::Expression),
    BranchStmt(branches::Branch),
    BreakStmt(branches::Interupter),
    ContinueStmt(branches::Interupter),
    ReturnStmt(branches::Interupter),
    Value(values::Value),
    TypeSpec(types::TypeSpec),
}

impl Ast {
    pub fn serialize(&self, writer: &mut XmlWriter) -> std::io::Result<()> {
        match self {
            Self::Scope(node) => node.serialize(writer),
            Self::ModuleDef(node) => node.serialize(writer),
            Self::StructDef(node) => node.serialize(writer),
            Self::VariantDef(node) => node.serialize(writer),
            Self::EnumDef(node) => node.serialize(writer),
            Self::FuncDef(node) => node.serialize(writer),
            Self::VariableDef(node) => node.serialize(writer),
            Self::Value(node) => node.serialize(writer),
            Self::TypeSpec(node) => node.serialize(writer),
            Self::AliasDef(node) => node.serialize(writer),
            Self::Expression(node) => node.serialize(writer),
            Self::BranchStmt(node) => node.serialize(writer),
            Self::BreakStmt(node) => node.serialize(writer),
            Self::ContinueStmt(node) => node.serialize(writer),
            Self::ReturnStmt(node) => node.serialize(writer),
        }
    }

    pub fn analyze(&mut self, analyzer: &mut Analyzer) -> Result<(), Message> {
        match self {
            Self::Scope(node) => node.analyze(analyzer),
            Self::FuncDef(node) => node.analyze(analyzer),
            Self::AliasDef(node) => node.analyze(analyzer),
            Self::ModuleDef(node) => node.analyze(analyzer),
            Self::StructDef(node) => node.analyze(analyzer),
            Self::EnumDef(node) => node.analyze(analyzer),
            Self::VariantDef(node) => node.analyze(analyzer),
            Self::VariableDef(node) => node.analyze(analyzer),
            Self::Value(node) => node.analyze(analyzer),
            Self::Expression(node) => node.analyze(analyzer),
            Self::BranchStmt(node) => node.analyze(analyzer),
            Self::ContinueStmt(node) => node.analyze(analyzer),
            Self::ReturnStmt(node) => node.analyze(analyzer),
            Self::BreakStmt(node) => node.analyze(analyzer),
            Self::TypeSpec(node) => node.analyze(analyzer),
        }?;

        // TODO: fix conversion
        // if matches!(self, Ast::Expression { .. }) {
        //     expressions::Expression::convert_ast_node(self, analyzer)?;
        // }

        Ok(())
    }

    pub fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        match self {
            Self::Scope(node) => node.codegen(stream),
            Self::ModuleDef(node) => node.codegen(stream),
            Self::StructDef(node) => node.codegen(stream),
            Self::EnumDef(node) => node.codegen(stream),
            Self::VariantDef(node) => node.codegen(stream),
            Self::FuncDef(node) => node.codegen(stream),
            Self::VariableDef(node) => node.codegen(stream),
            Self::Value(node) => node.codegen(stream),
            Self::TypeSpec(node) => node.codegen(stream),
            Self::AliasDef(node) => node.codegen(stream),
            Self::Expression(node) => node.codegen(stream),
            Self::BranchStmt(node) => node.codegen(stream),
            Self::BreakStmt(node) => node.codegen(stream),
            Self::ContinueStmt(node) => node.codegen(stream),
            Self::ReturnStmt(node) => node.codegen(stream),
        }
    }

    pub fn get_type(&self, analyzer: &mut Analyzer) -> Type {
        match self {
            Self::Expression(node) => node.get_type(analyzer),
            Self::AliasDef(node) => node.get_type(analyzer),
            Self::Value(node) => node.get_type(analyzer),
            Self::VariableDef(node) => node.get_type(analyzer),
            _ => todo!("GetType"),
        }
    }
}

pub fn analyze_program(ast: &mut Ast, analyzer: &mut Analyzer) -> Option<SymbolTable> {
    let res = ast.analyze(analyzer);

    if let Err(err) = &res {
        analyzer.error(err.clone());
    }

    if analyzer.has_errors() {
        None
    } else {
        Some(std::mem::take(&mut analyzer.table))
    }
}

pub fn parse_program(parser: &mut Parser) -> Option<Ast> {
    let res = scopes::Scope::parse_global(parser);

    if let Err(err) = &res {
        parser.error(err.clone());
    }

    if parser.has_errors() {
        None
    } else {
        Some(res.unwrap())
    }
}
