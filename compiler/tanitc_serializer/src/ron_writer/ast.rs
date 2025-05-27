use tanitc_ast::{
    AliasDef, Ast, Block, Branch, ControlFlow, EnumDef, Expression, ExternDef, FunctionDef,
    ModuleDef, StructDef, TypeSpec, UnionDef, Use, Value, VariableDef, VariantDef, Visitor,
};

use tanitc_lexer::location::Location;
use tanitc_messages::Message;

use super::RonWriter;

impl Visitor for RonWriter<'_> {
    fn visit_module_def(&mut self, module_def: &ModuleDef) -> Result<(), Message> {
        match self.serialize_module_def(module_def) {
            Ok(_) => Ok(()),
            Err(e) => Err(Self::serialize_err(e)),
        }
    }

    fn visit_struct_def(&mut self, struct_def: &StructDef) -> Result<(), Message> {
        match self.serialize_struct_def(struct_def) {
            Ok(_) => Ok(()),
            Err(e) => Err(Self::serialize_err(e)),
        }
    }

    fn visit_union_def(&mut self, union_def: &UnionDef) -> Result<(), Message> {
        match self.serialize_union_def(union_def) {
            Ok(_) => Ok(()),
            Err(e) => Err(Self::serialize_err(e)),
        }
    }

    fn visit_variant_def(&mut self, variant_def: &VariantDef) -> Result<(), Message> {
        match self.serialize_variant_def(variant_def) {
            Ok(_) => Ok(()),
            Err(e) => Err(Self::serialize_err(e)),
        }
    }

    fn visit_enum_def(&mut self, enum_def: &EnumDef) -> Result<(), Message> {
        match self.serialize_enum_def(enum_def) {
            Ok(_) => Ok(()),
            Err(e) => Err(Self::serialize_err(e)),
        }
    }

    fn visit_func_def(&mut self, func_def: &FunctionDef) -> Result<(), Message> {
        match self.serialize_func_def(func_def) {
            Ok(_) => Ok(()),
            Err(e) => Err(Self::serialize_err(e)),
        }
    }

    fn visit_extern_def(&mut self, extern_def: &ExternDef) -> Result<(), Message> {
        match self.serialize_extern_def(extern_def) {
            Ok(_) => Ok(()),
            Err(e) => Err(Self::serialize_err(e)),
        }
    }

    fn visit_variable_def(&mut self, var_def: &VariableDef) -> Result<(), Message> {
        match self.serialize_variable_def(var_def) {
            Ok(_) => Ok(()),
            Err(e) => Err(Self::serialize_err(e)),
        }
    }

    fn visit_alias_def(&mut self, alias_def: &AliasDef) -> Result<(), Message> {
        match self.serialize_alias_def(alias_def) {
            Ok(_) => Ok(()),
            Err(e) => Err(Self::serialize_err(e)),
        }
    }

    fn visit_expression(&mut self, expr: &Expression) -> Result<(), Message> {
        match self.serialize_expression(expr) {
            Ok(_) => Ok(()),
            Err(e) => Err(Self::serialize_err(e)),
        }
    }

    fn visit_branch(&mut self, branch: &Branch) -> Result<(), Message> {
        match self.serialize_branch(branch) {
            Ok(_) => Ok(()),
            Err(e) => Err(Self::serialize_err(e)),
        }
    }

    fn visit_control_flow(&mut self, cf: &ControlFlow) -> Result<(), Message> {
        match self.serialize_control_flow(cf) {
            Ok(_) => Ok(()),
            Err(e) => Err(Self::serialize_err(e)),
        }
    }

    fn visit_type_spec(&mut self, type_spec: &TypeSpec) -> Result<(), Message> {
        match self.serialize_type_spec(type_spec) {
            Ok(_) => Ok(()),
            Err(e) => Err(Self::serialize_err(e)),
        }
    }

    fn visit_use(&mut self, u: &Use) -> Result<(), Message> {
        match self.serialize_use(u) {
            Ok(_) => Ok(()),
            Err(e) => Err(Self::serialize_err(e)),
        }
    }

    fn visit_block(&mut self, block: &Block) -> Result<(), Message> {
        match self.serialize_block(block) {
            Ok(_) => Ok(()),
            Err(e) => Err(Self::serialize_err(e)),
        }
    }

    fn visit_value(&mut self, val: &Value) -> Result<(), Message> {
        match self.serialize_value(val) {
            Ok(_) => Ok(()),
            Err(e) => Err(Self::serialize_err(e)),
        }
    }
}

impl RonWriter<'_> {
    pub fn serialize(&mut self, ast: &Ast) -> Result<(), std::io::Error> {
        write!(self.stream, "{ast:#?}")
    }

    pub fn serialize_module_def(&mut self, module_def: &ModuleDef) -> Result<(), std::io::Error> {
        write!(self.stream, "{module_def:#?}")
    }

    pub fn serialize_struct_def(&mut self, struct_def: &StructDef) -> Result<(), std::io::Error> {
        write!(self.stream, "{struct_def:#?}")
    }

    pub fn serialize_union_def(&mut self, union_def: &UnionDef) -> Result<(), std::io::Error> {
        write!(self.stream, "{union_def:#?}")
    }

    pub fn serialize_variant_def(
        &mut self,
        variant_def: &VariantDef,
    ) -> Result<(), std::io::Error> {
        write!(self.stream, "{variant_def:#?}")
    }

    pub fn serialize_enum_def(&mut self, enum_def: &EnumDef) -> Result<(), std::io::Error> {
        write!(self.stream, "{enum_def:#?}")
    }

    pub fn serialize_func_def(&mut self, func_def: &FunctionDef) -> Result<(), std::io::Error> {
        write!(self.stream, "{func_def:#?}")
    }

    pub fn serialize_extern_def(&mut self, extern_def: &ExternDef) -> Result<(), std::io::Error> {
        write!(self.stream, "{extern_def:#?}")
    }

    pub fn serialize_variable_def(&mut self, var_def: &VariableDef) -> Result<(), std::io::Error> {
        write!(self.stream, "{var_def:#?}")
    }

    pub fn serialize_alias_def(&mut self, alias_def: &AliasDef) -> Result<(), std::io::Error> {
        write!(self.stream, "{alias_def:#?}")
    }

    pub fn serialize_expression(&mut self, expr: &Expression) -> Result<(), std::io::Error> {
        write!(self.stream, "{expr:#?}")
    }

    pub fn serialize_branch(&mut self, branch: &Branch) -> Result<(), std::io::Error> {
        write!(self.stream, "{branch:#?}")
    }

    pub fn serialize_control_flow(&mut self, cf: &ControlFlow) -> Result<(), std::io::Error> {
        write!(self.stream, "{cf:#?}")
    }

    pub fn serialize_type_spec(&mut self, type_spec: &TypeSpec) -> Result<(), std::io::Error> {
        write!(self.stream, "{type_spec:#?}")
    }

    pub fn serialize_use(&mut self, u: &Use) -> Result<(), std::io::Error> {
        write!(self.stream, "{u:#?}")
    }

    pub fn serialize_block(&mut self, block: &Block) -> Result<(), std::io::Error> {
        write!(self.stream, "{block:#?}")
    }

    pub fn serialize_value(&mut self, val: &Value) -> Result<(), std::io::Error> {
        write!(self.stream, "{val:#?}")
    }
}

impl RonWriter<'_> {
    fn serialize_err(e: std::io::Error) -> Message {
        Message {
            location: Location::default(),
            text: format!("RON serialization error: {e}"),
        }
    }
}
