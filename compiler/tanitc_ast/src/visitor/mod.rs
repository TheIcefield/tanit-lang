use tanitc_messages::Message;

use crate::ast::{
    aliases::AliasDef, blocks::Block, branches::Branch, control_flows::ControlFlow, enums::EnumDef,
    expressions::Expression, externs::ExternDef, functions::FunctionDef, methods::ImplDef,
    modules::ModuleDef, structs::StructDef, types::TypeSpec, unions::UnionDef, uses::Use,
    values::Value, variables::VariableDef, variants::VariantDef,
};

pub trait Visitor {
    fn visit_module_def(&mut self, module_def: &ModuleDef) -> Result<(), Message>;
    fn visit_struct_def(&mut self, struct_def: &StructDef) -> Result<(), Message>;
    fn visit_union_def(&mut self, union_def: &UnionDef) -> Result<(), Message>;
    fn visit_variant_def(&mut self, variant_def: &VariantDef) -> Result<(), Message>;
    fn visit_impl_def(&mut self, impl_def: &ImplDef) -> Result<(), Message>;
    fn visit_enum_def(&mut self, enum_def: &EnumDef) -> Result<(), Message>;
    fn visit_func_def(&mut self, func_def: &FunctionDef) -> Result<(), Message>;
    fn visit_extern_def(&mut self, extern_def: &ExternDef) -> Result<(), Message>;
    fn visit_variable_def(&mut self, var_def: &VariableDef) -> Result<(), Message>;
    fn visit_alias_def(&mut self, alias_def: &AliasDef) -> Result<(), Message>;
    fn visit_expression(&mut self, expr: &Expression) -> Result<(), Message>;
    fn visit_branch(&mut self, branch: &Branch) -> Result<(), Message>;
    fn visit_control_flow(&mut self, cf: &ControlFlow) -> Result<(), Message>;
    fn visit_type_spec(&mut self, type_spec: &TypeSpec) -> Result<(), Message>;
    fn visit_use(&mut self, u: &Use) -> Result<(), Message>;
    fn visit_block(&mut self, block: &Block) -> Result<(), Message>;
    fn visit_value(&mut self, val: &Value) -> Result<(), Message>;
}

pub trait VisitorMut {
    fn visit_module_def(&mut self, module_def: &mut ModuleDef) -> Result<(), Message>;
    fn visit_struct_def(&mut self, struct_def: &mut StructDef) -> Result<(), Message>;
    fn visit_union_def(&mut self, union_def: &mut UnionDef) -> Result<(), Message>;
    fn visit_variant_def(&mut self, variant_def: &mut VariantDef) -> Result<(), Message>;
    fn visit_impl_def(&mut self, impl_def: &mut ImplDef) -> Result<(), Message>;
    fn visit_enum_def(&mut self, enum_def: &mut EnumDef) -> Result<(), Message>;
    fn visit_func_def(&mut self, func_def: &mut FunctionDef) -> Result<(), Message>;
    fn visit_extern_def(&mut self, extern_def: &mut ExternDef) -> Result<(), Message>;
    fn visit_variable_def(&mut self, var_def: &mut VariableDef) -> Result<(), Message>;
    fn visit_alias_def(&mut self, alias_def: &mut AliasDef) -> Result<(), Message>;
    fn visit_expression(&mut self, expr: &mut Expression) -> Result<(), Message>;
    fn visit_branch(&mut self, branch: &mut Branch) -> Result<(), Message>;
    fn visit_control_flow(&mut self, cf: &mut ControlFlow) -> Result<(), Message>;
    fn visit_type_spec(&mut self, type_spec: &mut TypeSpec) -> Result<(), Message>;
    fn visit_use(&mut self, u: &mut Use) -> Result<(), Message>;
    fn visit_block(&mut self, block: &mut Block) -> Result<(), Message>;
    fn visit_value(&mut self, val: &mut Value) -> Result<(), Message>;
}
