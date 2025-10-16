use tanitc_ast::{
    ast::{
        aliases::AliasDef, blocks::Block, branches::Branch, control_flows::ControlFlow,
        enums::EnumDef, expressions::Expression, externs::ExternDef, functions::FunctionDef,
        methods::ImplDef, modules::ModuleDef, structs::StructDef, types::TypeSpec,
        unions::UnionDef, uses::Use, values::Value, variables::VariableDef, variants::VariantDef,
        Ast,
    },
    visitor::Visitor,
};
use tanitc_messages::Message;

use super::CodeGenStream;

mod aliases;
mod blocks;
mod branches;
mod control_flows;
mod enums;
mod expressions;
mod externs;
mod functions;
mod methods;
mod modules;
mod structs;
mod types;
mod unions;
mod uses;
mod values;
mod variables;
mod variants;

impl Visitor for CodeGenStream<'_> {
    fn visit_module_def(&mut self, module_def: &ModuleDef) -> Result<(), Message> {
        self.generate_module_def(module_def)
            .map_err(|err| Message::codegen_err(err, &module_def.location))
    }

    fn visit_struct_def(&mut self, struct_def: &StructDef) -> Result<(), Message> {
        self.generate_struct_def(struct_def)
            .map_err(|err| Message::codegen_err(err, &struct_def.location))
    }

    fn visit_union_def(&mut self, union_def: &UnionDef) -> Result<(), Message> {
        self.generate_union_def(union_def)
            .map_err(|err| Message::codegen_err(err, &union_def.location))
    }

    fn visit_variant_def(&mut self, variant_def: &VariantDef) -> Result<(), Message> {
        self.generate_variant_def(variant_def)
            .map_err(|err| Message::codegen_err(err, &variant_def.location))
    }

    fn visit_impl_def(&mut self, impl_def: &ImplDef) -> Result<(), Message> {
        self.generate_impl_def(impl_def)
            .map_err(|err| Message::codegen_err(err, &impl_def.location))
    }

    fn visit_enum_def(&mut self, enum_def: &EnumDef) -> Result<(), Message> {
        self.generate_enum_def(enum_def)
            .map_err(|err| Message::codegen_err(err, &enum_def.location))
    }

    fn visit_func_def(&mut self, func_def: &FunctionDef) -> Result<(), Message> {
        self.generate_func_def(func_def, None)
            .map_err(|err| Message::codegen_err(err, &func_def.location))
    }

    fn visit_extern_def(&mut self, extern_def: &ExternDef) -> Result<(), Message> {
        self.generate_extern_def(extern_def)
            .map_err(|err| Message::codegen_err(err, &extern_def.location))
    }

    fn visit_variable_def(&mut self, var_def: &VariableDef) -> Result<(), Message> {
        self.generate_variable_def(var_def)
            .map_err(|err| Message::codegen_err(err, &var_def.location))
    }

    fn visit_alias_def(&mut self, alias_def: &AliasDef) -> Result<(), Message> {
        self.generate_alias_def(alias_def)
            .map_err(|err| Message::codegen_err(err, &alias_def.location))
    }

    fn visit_expression(&mut self, expr: &Expression) -> Result<(), Message> {
        self.generate_expression(expr)
            .map_err(|err| Message::codegen_err(err, &expr.location))
    }

    fn visit_branch(&mut self, branch: &Branch) -> Result<(), Message> {
        self.generate_branch(branch)
            .map_err(|err| Message::codegen_err(err, &branch.location))
    }

    fn visit_control_flow(&mut self, cf: &ControlFlow) -> Result<(), Message> {
        self.generate_control_flow(cf)
            .map_err(|err| Message::codegen_err(err, &cf.location))
    }

    fn visit_type_spec(&mut self, type_spec: &TypeSpec) -> Result<(), Message> {
        self.generate_type_spec(type_spec)
            .map_err(|err| Message::codegen_err(err, &type_spec.location))
    }

    fn visit_use(&mut self, u: &Use) -> Result<(), Message> {
        self.generate_use(u)
            .map_err(|err| Message::codegen_err(err, &u.location))
    }

    fn visit_block(&mut self, block: &Block) -> Result<(), Message> {
        self.generate_block(block)
            .map_err(|err| Message::codegen_err(err, &block.location))
    }

    fn visit_value(&mut self, val: &Value) -> Result<(), Message> {
        self.generate_value(val)
            .map_err(|err| Message::codegen_err(err, &val.location))
    }
}

impl CodeGenStream<'_> {
    fn generate(&mut self, node: &Ast) -> Result<(), std::io::Error> {
        match node {
            Ast::ModuleDef(node) => self.generate_module_def(node),
            Ast::StructDef(node) => self.generate_struct_def(node),
            Ast::UnionDef(node) => self.generate_union_def(node),
            Ast::VariantDef(node) => self.generate_variant_def(node),
            Ast::ImplDef(node) => self.generate_impl_def(node),
            Ast::EnumDef(node) => self.generate_enum_def(node),
            Ast::FuncDef(node) => self.generate_func_def(node, None),
            Ast::VariableDef(node) => self.generate_variable_def(node),
            Ast::AliasDef(node) => self.generate_alias_def(node),
            Ast::ExternDef(node) => self.generate_extern_def(node),
            Ast::Expression(node) => self.generate_expression(node),
            Ast::BranchStmt(node) => self.generate_branch(node),
            Ast::ControlFlow(node) => self.generate_control_flow(node),
            Ast::TypeSpec(node) => self.generate_type_spec(node),
            Ast::Use(node) => self.generate_use(node),
            Ast::Block(node) => self.generate_block(node),
            Ast::Value(node) => self.generate_value(node),
        }
    }
}
