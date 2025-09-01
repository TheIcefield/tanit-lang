use tanitc_ast::{
    ast::{
        aliases::AliasDef, blocks::Block, branches::Branch, control_flows::ControlFlow,
        enums::EnumDef, expressions::Expression, externs::ExternDef, functions::FunctionDef,
        methods::ImplDef, modules::ModuleDef, structs::StructDef, types::TypeSpec,
        unions::UnionDef, uses::Use, values::Value, variables::VariableDef, variants::VariantDef,
        Ast,
    },
    visitor::VisitorMut,
};
use tanitc_messages::Message;
use tanitc_symbol_table::type_info::TypeInfo;

use crate::Analyzer;

pub mod aliases;
pub mod blocks;
pub mod branches;
pub mod control_flows;
pub mod enums;
pub mod expressions;
pub mod externs;
pub mod functions;
pub mod methods;
pub mod modules;
pub mod structs;
pub mod types;
pub mod unions;
pub mod values;
pub mod variables;
pub mod variants;

impl VisitorMut for Analyzer {
    fn visit_module_def(&mut self, module_def: &mut ModuleDef) -> Result<(), Message> {
        self.analyze_module_def(module_def)
    }

    fn visit_struct_def(&mut self, struct_def: &mut StructDef) -> Result<(), Message> {
        self.analyze_struct_def(struct_def)
    }

    fn visit_union_def(&mut self, union_def: &mut UnionDef) -> Result<(), Message> {
        self.analyze_union_def(union_def)
    }

    fn visit_variant_def(&mut self, variant_def: &mut VariantDef) -> Result<(), Message> {
        self.analyze_variant_def(variant_def)
    }

    fn visit_enum_def(&mut self, enum_def: &mut EnumDef) -> Result<(), Message> {
        self.analyze_enum_def(enum_def)
    }

    fn visit_impl_def(&mut self, impl_def: &mut ImplDef) -> Result<(), Message> {
        self.analyze_impl_def(impl_def)
    }

    fn visit_func_def(&mut self, func_def: &mut FunctionDef) -> Result<(), Message> {
        const NOT_METHOD: bool = false;

        self.analyze_func_def(func_def, NOT_METHOD)
    }

    fn visit_extern_def(&mut self, extern_def: &mut ExternDef) -> Result<(), Message> {
        self.analyze_extern_def(extern_def)
    }

    fn visit_variable_def(&mut self, var_def: &mut VariableDef) -> Result<(), Message> {
        self.analyze_variable_def(var_def)
    }

    fn visit_alias_def(&mut self, alias_def: &mut AliasDef) -> Result<(), Message> {
        self.analyze_alias_def(alias_def)
    }

    fn visit_expression(&mut self, expr: &mut Expression) -> Result<(), Message> {
        self.analyze_expression(expr)
    }

    fn visit_branch(&mut self, branch: &mut Branch) -> Result<(), Message> {
        self.analyze_branch(branch)
    }

    fn visit_control_flow(&mut self, cf: &mut ControlFlow) -> Result<(), Message> {
        self.analyze_control_flow(cf)
    }

    fn visit_type_spec(&mut self, _type_spec: &mut TypeSpec) -> Result<(), Message> {
        Ok(())
    }

    fn visit_use(&mut self, _u: &mut Use) -> Result<(), Message> {
        Ok(())
    }

    fn visit_block(&mut self, block: &mut Block) -> Result<(), Message> {
        self.analyze_block(block)
    }

    fn visit_value(&mut self, value: &mut Value) -> Result<(), Message> {
        self.analyze_value(value)
    }
}

impl Analyzer {
    pub fn get_type(&self, node: &Ast) -> TypeInfo {
        match node {
            Ast::AliasDef(node) => self.get_alias_def_type(node),
            Ast::VariableDef(node) => self.get_var_def_type(node),
            Ast::Expression(node) => self.get_expr_type(node),
            Ast::Value(node) => self.get_value_type(node),
            _ => todo!("GetType: {}", node.name()),
        }
    }
}
