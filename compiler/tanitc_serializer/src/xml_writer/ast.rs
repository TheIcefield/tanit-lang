use tanitc_ast::{
    attributes, AliasDef, Block, Branch, BranchKind, CallArgKind, ControlFlow, ControlFlowKind,
    EnumDef, Expression, ExpressionKind, ExternDef, FieldInfo, FunctionDef, ModuleDef, StructDef,
    TypeInfo, TypeSpec, UnionDef, Use, UseIdentifier, Value, ValueKind, VariableDef, VariantDef,
    VariantField, Visitor,
};
use tanitc_ident::Ident;
use tanitc_messages::Message;
use tanitc_ty::{ArraySize, Type};

use super::XmlWriter;

impl Visitor for XmlWriter<'_> {
    fn visit_module_def(&mut self, module_def: &ModuleDef) -> Result<(), Message> {
        if module_def.body.is_some() {
            self.serializer_module_def_internal(module_def)
        } else {
            self.serializer_module_def_external(module_def)
        }
    }

    fn visit_struct_def(&mut self, struct_def: &StructDef) -> Result<(), Message> {
        self.begin_tag("struct-definition")?;
        self.put_param("name", struct_def.identifier)?;

        self.serialize_struct_attributes(&struct_def.attributes)?;

        for internal in struct_def.internals.iter() {
            internal.accept(self)?;
        }

        for (field_id, field_info) in struct_def.fields.iter() {
            self.serialize_field_info(*field_id, field_info)?;
        }

        self.end_tag()?;

        Ok(())
    }

    fn visit_union_def(&mut self, union_def: &UnionDef) -> Result<(), Message> {
        self.begin_tag("union-definition")?;
        self.put_param("name", union_def.identifier)?;

        self.serialize_union_attributes(&union_def.attributes)?;

        for internal in union_def.internals.iter() {
            internal.accept(self)?;
        }

        for (field_id, field_info) in union_def.fields.iter() {
            self.serialize_field_info(*field_id, field_info)?;
        }

        self.end_tag()?;

        Ok(())
    }

    fn visit_variant_def(&mut self, variant_def: &VariantDef) -> Result<(), Message> {
        self.begin_tag("variant-definition")?;
        self.put_param("name", variant_def.identifier)?;

        self.serialize_variant_attributes(&variant_def.attributes)?;

        for internal in variant_def.internals.iter() {
            internal.accept(self)?;
        }

        for (field_id, field) in variant_def.fields.iter() {
            self.begin_tag("field")?;
            self.put_param("name", field_id)?;

            if VariantField::Common == *field {
                self.end_tag()?;
                continue;
            }

            self.serialize_variant_field(field)?;

            self.end_tag()?;
        }

        self.end_tag()?;

        Ok(())
    }

    fn visit_enum_def(&mut self, enum_def: &EnumDef) -> Result<(), Message> {
        self.begin_tag("enum-definition")?;
        self.put_param("name", enum_def.identifier)?;

        self.serialize_enum_attributes(&enum_def.attributes)?;

        for field in enum_def.fields.iter() {
            self.begin_tag("field")?;
            self.put_param("name", field.0)?;

            if let Some(value) = &field.1 {
                self.put_param("value", value)?;
            }

            self.end_tag()?;
        }

        self.end_tag()?;

        Ok(())
    }

    fn visit_func_def(&mut self, func_def: &FunctionDef) -> Result<(), Message> {
        self.begin_tag("function-definition")?;
        self.put_param("name", func_def.identifier)?;

        self.serialize_func_attributes(&func_def.attributes)?;

        self.begin_tag("return-type")?;
        self.visit_type_spec(&func_def.return_type)?;
        self.end_tag()?;

        if !func_def.parameters.is_empty() {
            self.begin_tag("parameters")?;
            for param in func_def.parameters.iter() {
                param.accept(self)?;
            }
            self.end_tag()?;
        }

        if let Some(body) = &func_def.body {
            body.accept(self)?;
        }

        self.end_tag()?;

        Ok(())
    }

    fn visit_extern_def(&mut self, extern_def: &ExternDef) -> Result<(), Message> {
        self.begin_tag("extern-definition")?;

        self.put_param("abi-name", &extern_def.abi_name)?;

        for func_def in extern_def.functions.iter() {
            self.visit_func_def(func_def)?;
        }

        self.end_tag()?;

        Ok(())
    }

    fn visit_variable_def(&mut self, var_def: &VariableDef) -> Result<(), Message> {
        self.begin_tag("variable-definition")?;
        self.put_param("name", var_def.identifier)?;

        self.serialize_variable_attributes(&var_def.attributes)?;

        self.put_param("is-global", var_def.is_global)?;
        self.put_param("is-mutable", var_def.is_mutable)?;

        self.visit_type_spec(&var_def.var_type)?;

        self.end_tag()?;

        Ok(())
    }

    fn visit_alias_def(&mut self, alias_def: &AliasDef) -> Result<(), Message> {
        self.begin_tag("alias-definition")?;
        self.put_param("name", alias_def.identifier)?;

        self.serialize_alias_attributes(&alias_def.attributes)?;

        self.visit_type_spec(&alias_def.value)?;

        self.end_tag()?;

        Ok(())
    }

    fn visit_expression(&mut self, expr: &Expression) -> Result<(), Message> {
        self.begin_tag("operation")?;

        match &expr.kind {
            ExpressionKind::Unary { operation, node } => {
                self.put_param("style", "unary")?;
                self.put_param("operation", operation)?;
                node.accept(self)?;
            }
            ExpressionKind::Binary {
                operation,
                lhs,
                rhs,
            } => {
                self.put_param("style", "binary")?;
                self.put_param("operation", operation)?;

                lhs.accept(self)?;
                rhs.accept(self)?;
            }
            ExpressionKind::Conversion { lhs, ty } => {
                self.put_param("style", "conversion")?;

                self.visit_type_spec(ty)?;
                lhs.accept(self)?;
            }
            ExpressionKind::Access { lhs, rhs } => {
                self.put_param("style", "access")?;

                lhs.accept(self)?;
                rhs.accept(self)?;
            }
            ExpressionKind::Get { lhs, rhs } => {
                self.put_param("style", "get")?;

                lhs.accept(self)?;
                rhs.accept(self)?;
            }
            ExpressionKind::Indexing { lhs, index } => {
                self.put_param("style", "indexing")?;

                lhs.accept(self)?;
                index.accept(self)?;
            }
            ExpressionKind::Term { node, .. } => {
                node.accept(self)?;
            }
        }

        self.end_tag()?;

        Ok(())
    }

    fn visit_branch(&mut self, branch: &Branch) -> Result<(), Message> {
        match &branch.kind {
            BranchKind::While { body, condition } => {
                self.begin_tag("while")?;

                self.begin_tag("condition")?;
                condition.accept(self)?;
                self.end_tag()?;

                body.accept(self)?;

                self.end_tag()?;
            }
            BranchKind::Loop { body } => {
                self.begin_tag("loop")?;

                body.accept(self)?;

                self.end_tag()?;
            }
            BranchKind::If { condition, body } => {
                self.begin_tag("if")?;

                self.begin_tag("condition")?;
                condition.accept(self)?;
                self.end_tag()?;

                self.begin_tag("than")?;
                body.accept(self)?;
                self.end_tag()?;

                self.end_tag()?;
            }
            BranchKind::Else { body } => {
                self.begin_tag("else")?;

                body.accept(self)?;

                self.end_tag()?;
            }
        }

        Ok(())
    }

    fn visit_control_flow(&mut self, cf: &ControlFlow) -> Result<(), Message> {
        self.begin_tag(&format!("{}-statement", cf.kind.to_str()))?;

        match &cf.kind {
            ControlFlowKind::Break { ret } | ControlFlowKind::Return { ret } => {
                if let Some(expr) = ret {
                    expr.accept(self)?;
                }
            }
            _ => {}
        }

        self.end_tag()?;

        Ok(())
    }

    fn visit_type_spec(&mut self, type_spec: &TypeSpec) -> Result<(), Message> {
        self.begin_tag("type")?;

        self.serialize_type(&type_spec.ty, type_spec.info)?;

        self.end_tag()?;

        Ok(())
    }

    fn visit_use(&mut self, u: &Use) -> Result<(), Message> {
        self.begin_tag("use")?;

        let mut param = use_identifier_to_str(&u.identifier[0])?;
        for i in u.identifier.iter().skip(1) {
            param.push_str("::");
            param.push_str(use_identifier_to_str(i)?.as_str());
        }

        self.put_param("name", param)?;

        self.end_tag()?;

        Ok(())
    }

    fn visit_block(&mut self, block: &Block) -> Result<(), Message> {
        let is_default = block.attributes == attributes::BlockAttributes::default();

        if !is_default {
            self.begin_tag("block")?;
            self.serialize_block_attributes(&block.attributes)?;
        }

        for stmt in block.statements.iter() {
            stmt.accept(self)?;
        }

        if !is_default {
            self.end_tag()?;
        }

        Ok(())
    }

    fn visit_value(&mut self, val: &Value) -> Result<(), Message> {
        match &val.kind {
            ValueKind::Call {
                identifier,
                arguments,
            } => {
                self.begin_tag("call-statement")?;
                self.put_param("name", identifier)?;

                if !arguments.is_empty() {
                    self.begin_tag("parameters")?;
                    for arg in arguments.iter() {
                        self.begin_tag("parameter")?;
                        match &arg.kind {
                            CallArgKind::Notified(id, expr) => {
                                self.put_param("name", id)?;
                                expr.accept(self)?;
                            }
                            CallArgKind::Positional(index, expr) => {
                                self.put_param("index", index)?;
                                expr.accept(self)?;
                            }
                        }
                        self.end_tag()?; //parameter
                    }
                    self.end_tag()?; // parameters
                }

                self.end_tag()?; // call-statement
            }
            ValueKind::Struct {
                identifier,
                components,
            } => {
                self.begin_tag("struct-initialization")?;
                self.put_param("name", identifier)?;

                for (comp_id, comp_type) in components.iter() {
                    self.begin_tag("field")?;
                    self.put_param("name", comp_id)?;

                    comp_type.accept(self)?;

                    self.end_tag()?;
                }

                self.end_tag()?;
            }
            ValueKind::Tuple { components } => {
                self.begin_tag("tuple-initialization")?;

                for component in components.iter() {
                    component.accept(self)?;
                }

                self.end_tag()?;
            }
            ValueKind::Array { components } => {
                self.begin_tag("array-initialization")?;

                for component in components.iter() {
                    component.accept(self)?;
                }

                self.end_tag()?;
            }
            ValueKind::Identifier(id) => {
                self.begin_tag("identifier")?;
                self.put_param("name", id)?;
                self.end_tag()?;
            }
            ValueKind::Text(value) => {
                self.begin_tag("literal")?;
                self.put_param("style", "text")?;
                self.put_param("value", value)?;
                self.end_tag()?;
            }
            ValueKind::Integer(value) => {
                self.begin_tag("literal")?;
                self.put_param("style", "integer-number")?;
                self.put_param("value", value)?;
                self.end_tag()?;
            }
            ValueKind::Decimal(value) => {
                self.begin_tag("literal")?;
                self.put_param("style", "decimal-number")?;
                self.put_param("value", value)?;
                self.end_tag()?;
            }
        }

        Ok(())
    }
}

// Module definition
impl XmlWriter<'_> {
    fn serializer_module_def_internal(&mut self, module_def: &ModuleDef) -> Result<(), Message> {
        self.begin_tag("module-definition")?;
        self.put_param("name", module_def.identifier)?;

        self.serialize_module_attributes(&module_def.attributes)?;

        if let Some(body) = &module_def.body {
            self.visit_block(body)?;
        }

        self.end_tag()?;

        Ok(())
    }

    fn serializer_module_def_external(&mut self, module_def: &ModuleDef) -> Result<(), Message> {
        self.begin_tag("module-import")?;
        self.put_param("name", module_def.identifier)?;

        self.serialize_module_attributes(&module_def.attributes)?;

        self.end_tag()?;

        Ok(())
    }
}

impl XmlWriter<'_> {
    fn serialize_variant_field(&mut self, field: &VariantField) -> Result<(), Message> {
        match field {
            VariantField::StructLike(s) => {
                for (field_id, field_type) in s.iter() {
                    self.begin_tag("field")?;
                    self.put_param("name", field_id)?;

                    self.visit_type_spec(field_type)?;

                    self.end_tag()?;
                }
            }
            VariantField::TupleLike(tuple_field) => {
                for tuple_component in tuple_field.iter() {
                    self.visit_type_spec(tuple_component)?;
                }
            }
            _ => {}
        }

        Ok(())
    }
}

impl XmlWriter<'_> {
    fn serialize_type(&mut self, ty: &Type, info: TypeInfo) -> Result<(), Message> {
        match ty {
            Type::Ref { ref_to, is_mutable } => {
                self.put_param("style", "reference")?;
                self.put_param("is-mutable", is_mutable)?;

                self.serialize_type(ref_to, info)?;
            }
            Type::Ptr(ptr_to) => {
                self.put_param("style", "pointer")?;
                self.put_param("is-mutable", info.is_mut)?;

                self.serialize_type(ptr_to, info)?;
            }
            Type::Tuple(components) => {
                self.put_param("style", "tuple")?;

                for comp in components.iter() {
                    self.serialize_type(comp, info)?;
                }
            }
            Type::Array { size, value_type } => {
                self.put_param("style", "array")?;

                if let ArraySize::Fixed(size) = size {
                    self.put_param("size", size)?;
                }

                self.serialize_type(value_type, info)?;
            }
            Type::Template {
                identifier,
                generics,
            } => {
                self.put_param("style", "generic")?;
                self.put_param("name", identifier)?;

                for generic in generics.iter() {
                    self.serialize_type(generic, info)?;
                }
            }
            Type::Custom(id) => {
                self.put_param("style", "named")?;
                self.put_param("name", id)?
            }
            Type::Auto => self.put_param("style", "automatic")?,
            Type::Never => {
                self.put_param("style", "never")?;
            }
            Type::Bool => {
                self.put_param("style", "primitive")?;
                self.put_param("name", "bool")?;
            }
            Type::I8 => {
                self.put_param("style", "primitive")?;
                self.put_param("name", "i8")?;
            }
            Type::I16 => {
                self.put_param("style", "primitive")?;
                self.put_param("name", "i16")?;
            }
            Type::I32 => {
                self.put_param("style", "primitive")?;
                self.put_param("name", "i32")?;
            }
            Type::I64 => {
                self.put_param("style", "primitive")?;
                self.put_param("name", "i64")?;
            }
            Type::I128 => {
                self.put_param("style", "primitive")?;
                self.put_param("name", "i128")?;
            }
            Type::U8 => {
                self.put_param("style", "primitive")?;
                self.put_param("name", "u8")?;
            }
            Type::U16 => {
                self.put_param("style", "primitive")?;
                self.put_param("name", "u16")?;
            }
            Type::U32 => {
                self.put_param("style", "primitive")?;
                self.put_param("name", "u32")?;
            }
            Type::U64 => {
                self.put_param("style", "primitive")?;
                self.put_param("name", "u64")?;
            }
            Type::U128 => {
                self.put_param("style", "primitive")?;
                self.put_param("name", "u128")?;
            }
            Type::F32 => {
                self.put_param("style", "primitive")?;
                self.put_param("name", "f32")?;
            }
            Type::F64 => {
                self.put_param("style", "primitive")?;
                self.put_param("name", "f64")?;
            }
            Type::Str => {
                self.put_param("style", "primitive")?;
                self.put_param("name", "str")?;
            }
        }

        Ok(())
    }
}

impl XmlWriter<'_> {
    fn serialize_field_info(
        &mut self,
        field_id: Ident,
        field_info: &FieldInfo,
    ) -> Result<(), Message> {
        self.begin_tag("field")?;
        self.put_param("name", field_id)?;
        self.serialize_publicity(&field_info.attributes.publicity)?;

        self.visit_type_spec(&field_info.ty)?;

        self.end_tag()?;

        Ok(())
    }
}

// Attributes
impl XmlWriter<'_> {
    fn serialize_safety(&mut self, safety: &attributes::Safety) -> Result<(), Message> {
        self.put_param("safety", safety)?;

        Ok(())
    }

    fn serialize_publicity(&mut self, publicity: &attributes::Publicity) -> Result<(), Message> {
        self.put_param("publicity", publicity)?;

        Ok(())
    }

    fn serialize_block_attributes(
        &mut self,
        attrs: &attributes::BlockAttributes,
    ) -> Result<(), Message> {
        if *attrs == attributes::BlockAttributes::default() {
            return Ok(());
        }

        self.begin_tag("attributes")?;

        self.serialize_safety(&attrs.safety)?;

        self.end_tag()?;

        Ok(())
    }

    fn serialize_func_attributes(
        &mut self,
        attrs: &attributes::FunctionAttributes,
    ) -> Result<(), Message> {
        if *attrs == attributes::FunctionAttributes::default() {
            return Ok(());
        }

        self.begin_tag("attributes")?;

        self.serialize_safety(&attrs.safety)?;
        self.serialize_publicity(&attrs.publicity)?;

        self.end_tag()?;

        Ok(())
    }

    fn serialize_module_attributes(
        &mut self,
        attrs: &attributes::ModuleAttributes,
    ) -> Result<(), Message> {
        if *attrs == attributes::ModuleAttributes::default() {
            return Ok(());
        }

        self.begin_tag("attributes")?;

        self.serialize_safety(&attrs.safety)?;
        self.serialize_publicity(&attrs.publicity)?;

        self.end_tag()?;

        Ok(())
    }

    fn serialize_struct_attributes(
        &mut self,
        attrs: &attributes::StructAttributes,
    ) -> Result<(), Message> {
        if *attrs == attributes::StructAttributes::default() {
            return Ok(());
        }

        self.begin_tag("attributes")?;

        self.serialize_publicity(&attrs.publicity)?;

        self.end_tag()?;

        Ok(())
    }

    fn serialize_union_attributes(
        &mut self,
        attrs: &attributes::UnionAttributes,
    ) -> Result<(), Message> {
        if *attrs == attributes::UnionAttributes::default() {
            return Ok(());
        }

        self.begin_tag("attributes")?;

        self.serialize_publicity(&attrs.publicity)?;

        self.end_tag()?;

        Ok(())
    }

    fn serialize_enum_attributes(
        &mut self,
        attrs: &attributes::EnumAttributes,
    ) -> Result<(), Message> {
        if *attrs == attributes::EnumAttributes::default() {
            return Ok(());
        }

        self.begin_tag("attributes")?;

        self.serialize_publicity(&attrs.publicity)?;

        self.end_tag()?;

        Ok(())
    }

    fn serialize_alias_attributes(
        &mut self,
        attrs: &attributes::AliasAttributes,
    ) -> Result<(), Message> {
        if *attrs == attributes::AliasAttributes::default() {
            return Ok(());
        }

        self.begin_tag("attributes")?;

        self.serialize_publicity(&attrs.publicity)?;

        self.end_tag()?;

        Ok(())
    }

    fn serialize_variant_attributes(
        &mut self,
        attrs: &attributes::VariantAttributes,
    ) -> Result<(), Message> {
        if *attrs == attributes::VariantAttributes::default() {
            return Ok(());
        }

        self.begin_tag("attributes")?;

        self.serialize_publicity(&attrs.publicity)?;

        self.end_tag()?;

        Ok(())
    }

    fn serialize_variable_attributes(
        &mut self,
        attrs: &attributes::VariableAttributes,
    ) -> Result<(), Message> {
        if *attrs == attributes::VariableAttributes::default() {
            return Ok(());
        }

        self.begin_tag("attributes")?;

        self.serialize_publicity(&attrs.publicity)?;

        self.end_tag()?;

        Ok(())
    }
}

fn use_identifier_to_str(id: &UseIdentifier) -> Result<String, Message> {
    match id {
        UseIdentifier::BuiltInSelf => Ok(String::from("Self")),
        UseIdentifier::BuiltInSuper => Ok(String::from("Super")),
        UseIdentifier::BuiltInCrate => Ok(String::from("Crate")),
        UseIdentifier::BuiltInAll => Ok(String::from("*")),
        UseIdentifier::Identifier(id) => Ok(String::from(*id)),
    }
}
