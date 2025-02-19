use tanitc_ast::{
    AliasDef, AstVisitor, Block, Branch, BranchKind, CallParam, ControlFlow, ControlFlowKind,
    EnumDef, Expression, ExpressionKind, FunctionDef, ModuleDef, StructDef, TypeInfo, TypeSpec,
    Value, ValueKind, VariableDef, VariantDef, VariantField,
};
use tanitc_messages::Message;
use tanitc_ty::Type;

use super::XmlWriter;

impl AstVisitor for XmlWriter<'_> {
    fn visit_module_def(&mut self, module_def: &mut ModuleDef) -> Result<(), Message> {
        if module_def.body.is_some() {
            self.serializer_module_def_internal(module_def)
        } else {
            self.serializer_module_def_external(module_def)
        }
    }

    fn visit_struct_def(&mut self, struct_def: &mut StructDef) -> Result<(), Message> {
        self.begin_tag("struct-definition")?;
        self.put_param("name", struct_def.identifier)?;

        for internal in struct_def.internals.iter_mut() {
            self.visit(internal)?;
        }

        for (field_id, field_type) in struct_def.fields.iter_mut() {
            self.begin_tag("field")?;
            self.put_param("name", field_id)?;

            self.visit_type_spec(field_type)?;

            self.end_tag()?;
        }

        self.end_tag()?;

        Ok(())
    }

    fn visit_variant_def(&mut self, variant_def: &mut VariantDef) -> Result<(), Message> {
        self.begin_tag("variant-definition")?;
        self.put_param("name", variant_def.identifier)?;

        for internal in variant_def.internals.iter_mut() {
            self.visit(internal)?;
        }

        for (field_id, field) in variant_def.fields.iter_mut() {
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

    fn visit_enum_def(&mut self, enum_def: &mut EnumDef) -> Result<(), Message> {
        self.begin_tag("enum-definition")?;
        self.put_param("name", enum_def.identifier)?;

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

    fn visit_func_def(&mut self, func_def: &mut FunctionDef) -> Result<(), Message> {
        self.begin_tag("function-definition")?;
        self.put_param("name", func_def.identifier)?;

        self.begin_tag("return-type")?;
        self.visit_type_spec(&mut func_def.return_type)?;
        self.end_tag()?;

        if !func_def.parameters.is_empty() {
            self.begin_tag("parameters")?;
            for param in func_def.parameters.iter_mut() {
                self.visit(param)?;
            }
            self.end_tag()?;
        }

        if let Some(body) = &mut func_def.body {
            self.visit(body)?;
        }

        self.end_tag()?;

        Ok(())
    }

    fn visit_variable_def(&mut self, var_def: &mut VariableDef) -> Result<(), Message> {
        self.begin_tag("variable-definition")?;
        self.put_param("name", var_def.identifier)?;
        self.put_param("is-global", var_def.is_global)?;
        self.put_param("is-mutable", var_def.is_mutable)?;

        self.visit_type_spec(&mut var_def.var_type)?;

        self.end_tag()?;

        Ok(())
    }

    fn visit_alias_def(&mut self, alias_def: &mut AliasDef) -> Result<(), Message> {
        self.begin_tag("alias-definition")?;
        self.put_param("name", alias_def.identifier)?;

        self.visit_type_spec(&mut alias_def.value)?;

        self.end_tag()?;

        Ok(())
    }

    fn visit_expression(&mut self, expr: &mut Expression) -> Result<(), Message> {
        self.begin_tag("operation")?;

        match &mut expr.kind {
            ExpressionKind::Unary { operation, node } => {
                self.put_param("style", "unary")?;
                self.put_param("operation", operation)?;
                self.visit(node)?;
            }
            ExpressionKind::Binary {
                operation,
                lhs,
                rhs,
            } => {
                self.put_param("style", "binary")?;
                self.put_param("operation", operation)?;

                self.visit(lhs)?;
                self.visit(rhs)?;
            }
            ExpressionKind::Conversion { lhs, ty } => {
                self.put_param("style", "conversion")?;

                self.visit_type_spec(ty)?;
                self.visit(lhs)?;
            }
        }

        self.end_tag()?;

        Ok(())
    }

    fn visit_branch(&mut self, branch: &mut Branch) -> Result<(), Message> {
        match &mut branch.kind {
            BranchKind::While { body, condition } => {
                self.begin_tag("while")?;

                self.begin_tag("condition")?;
                self.visit(condition)?;
                self.end_tag()?;

                self.visit(body)?;

                self.end_tag()?;
            }
            BranchKind::Loop { body } => {
                self.begin_tag("loop")?;

                self.visit(body)?;

                self.end_tag()?;
            }
            BranchKind::If { condition, body } => {
                self.begin_tag("if")?;

                self.begin_tag("condition")?;
                self.visit(condition)?;
                self.end_tag()?;

                self.begin_tag("than")?;
                self.visit(body)?;
                self.end_tag()?;

                self.end_tag()?;
            }
            BranchKind::Else { body } => {
                self.begin_tag("else")?;
                self.visit(body)?;
                self.end_tag()?;
            }
        }

        Ok(())
    }

    fn visit_control_flow(&mut self, cf: &mut ControlFlow) -> Result<(), Message> {
        self.begin_tag(&format!("{}-statement", cf.kind.to_str()))?;

        match &mut cf.kind {
            ControlFlowKind::Break { ret } | ControlFlowKind::Return { ret } => {
                if let Some(expr) = ret {
                    self.visit(expr)?;
                }
            }
            _ => {}
        }

        self.end_tag()?;

        Ok(())
    }

    fn visit_type_spec(&mut self, type_spec: &mut TypeSpec) -> Result<(), Message> {
        self.begin_tag("type")?;

        self.serialize_type(&type_spec.ty, type_spec.info)?;

        self.end_tag()?;

        Ok(())
    }

    fn visit_block(&mut self, block: &mut Block) -> Result<(), Message> {
        for stmt in block.statements.iter_mut() {
            self.visit(stmt)?;
        }

        Ok(())
    }

    fn visit_value(&mut self, val: &mut Value) -> Result<(), Message> {
        match &mut val.kind {
            ValueKind::Call {
                identifier,
                arguments,
            } => {
                self.begin_tag("call-statement")?;
                self.put_param("name", identifier)?;

                self.begin_tag("parameters")?;
                for arg in arguments.iter_mut() {
                    self.begin_tag("parameter")?;
                    match arg {
                        CallParam::Notified(id, expr) => {
                            self.put_param("name", id)?;
                            self.visit(expr.as_mut())?;
                        }
                        CallParam::Positional(index, expr) => {
                            self.put_param("index", index)?;
                            self.visit(expr.as_mut())?;
                        }
                    }
                    self.end_tag()?; //parameter
                }
                self.end_tag()?; // parameters
                self.end_tag()?; // call-statement
            }
            ValueKind::Struct {
                identifier,
                components,
            } => {
                self.begin_tag("struct-initialization")?;
                self.put_param("name", identifier)?;

                for (comp_id, comp_type) in components.iter_mut() {
                    self.begin_tag("field")?;
                    self.put_param("name", comp_id)?;

                    self.visit(comp_type)?;

                    self.end_tag()?;
                }

                self.end_tag()?;
            }
            ValueKind::Tuple { components } => {
                self.begin_tag("tuple-initialization")?;

                for component in components.iter_mut() {
                    self.visit(component)?
                }

                self.end_tag()?;
            }
            ValueKind::Array { components } => {
                self.begin_tag("array-initialization")?;

                for component in components.iter_mut() {
                    self.visit(component)?
                }

                self.end_tag()?;
            }
            ValueKind::Identifier(id) => {
                self.begin_tag("variable")?;
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

impl XmlWriter<'_> {
    fn serializer_module_def_internal(
        &mut self,
        module_def: &mut ModuleDef,
    ) -> Result<(), Message> {
        self.begin_tag("module-definition")?;
        self.put_param("name", module_def.identifier)?;

        if let Some(body) = &mut module_def.body {
            self.visit_block(body)?;
        }

        self.end_tag()?;

        Ok(())
    }

    fn serializer_module_def_external(
        &mut self,
        module_def: &mut ModuleDef,
    ) -> Result<(), Message> {
        self.begin_tag("module-import")?;
        self.put_param("name", module_def.identifier)?;

        self.end_tag()?;

        Ok(())
    }
}

impl XmlWriter<'_> {
    fn serialize_variant_field(&mut self, field: &mut VariantField) -> Result<(), Message> {
        match field {
            VariantField::StructLike(s) => {
                for (field_id, field_type) in s.iter_mut() {
                    self.begin_tag("field")?;
                    self.put_param("name", field_id)?;

                    self.visit_type_spec(field_type)?;

                    self.end_tag()?;
                }
            }
            VariantField::TupleLike(tuple_field) => {
                for tuple_component in tuple_field.iter_mut() {
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
            Type::Ref(ref_to) => {
                self.put_param("style", "reference")?;
                self.put_param("is-mutable", info.is_mut)?;

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

                if let Some(size) = size {
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
