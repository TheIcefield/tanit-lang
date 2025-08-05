use tanitc_ast::{
    ast::{
        aliases::AliasDef,
        blocks::Block,
        branches::Branch,
        control_flows::ControlFlow,
        enums::EnumDef,
        expressions::{BinaryOperation, Expression, ExpressionKind, UnaryOperation},
        externs::ExternDef,
        functions::FunctionDef,
        methods::ImplDef,
        modules::ModuleDef,
        structs::StructDef,
        types::TypeSpec,
        unions::UnionDef,
        uses::Use,
        values::{CallArg, CallArgKind, Value, ValueKind},
        variables::VariableDef,
        variants::VariantDef,
        Ast,
    },
    visitor::Visitor,
};
use tanitc_lexer::location::Location;
use tanitc_messages::Message;
use tanitc_ty::{ArraySize, Type};

use super::{CodeGenMode, CodeGenStream};

use std::io::Write;

mod aliases;
mod blocks;
mod branches;
mod control_flows;
mod enums;
mod externs;
mod functions;
mod methods;
mod modules;
mod structs;
mod unions;
mod uses;
mod variants;

impl Visitor for CodeGenStream<'_> {
    fn visit_module_def(&mut self, module_def: &ModuleDef) -> Result<(), Message> {
        match self.generate_module_def(module_def) {
            Ok(_) => Ok(()),
            Err(e) => Err(Self::codegen_err(e)),
        }
    }

    fn visit_struct_def(&mut self, struct_def: &StructDef) -> Result<(), Message> {
        match self.generate_struct_def(struct_def) {
            Ok(_) => Ok(()),
            Err(e) => Err(Self::codegen_err(e)),
        }
    }

    fn visit_union_def(&mut self, union_def: &UnionDef) -> Result<(), Message> {
        match self.generate_union_def(union_def) {
            Ok(_) => Ok(()),
            Err(e) => Err(Self::codegen_err(e)),
        }
    }

    fn visit_variant_def(&mut self, variant_def: &VariantDef) -> Result<(), Message> {
        match self.generate_variant_def(variant_def) {
            Ok(_) => Ok(()),
            Err(e) => Err(Self::codegen_err(e)),
        }
    }

    fn visit_impl_def(&mut self, impl_def: &ImplDef) -> Result<(), Message> {
        match self.generate_impl_def(impl_def) {
            Ok(_) => Ok(()),
            Err(e) => Err(Self::codegen_err(e)),
        }
    }

    fn visit_enum_def(&mut self, enum_def: &EnumDef) -> Result<(), Message> {
        match self.generate_enum_def(enum_def) {
            Ok(_) => Ok(()),
            Err(e) => Err(Self::codegen_err(e)),
        }
    }

    fn visit_func_def(&mut self, func_def: &FunctionDef) -> Result<(), Message> {
        match self.generate_func_def(func_def, None) {
            Ok(_) => Ok(()),
            Err(e) => Err(Self::codegen_err(e)),
        }
    }

    fn visit_extern_def(&mut self, extern_def: &ExternDef) -> Result<(), Message> {
        match self.generate_extern_def(extern_def) {
            Ok(_) => Ok(()),
            Err(e) => Err(Self::codegen_err(e)),
        }
    }

    fn visit_variable_def(&mut self, var_def: &VariableDef) -> Result<(), Message> {
        match self.generate_variable_def(var_def) {
            Ok(_) => Ok(()),
            Err(e) => Err(Self::codegen_err(e)),
        }
    }

    fn visit_alias_def(&mut self, alias_def: &AliasDef) -> Result<(), Message> {
        match self.generate_alias_def(alias_def) {
            Ok(_) => Ok(()),
            Err(e) => Err(Self::codegen_err(e)),
        }
    }

    fn visit_expression(&mut self, expr: &Expression) -> Result<(), Message> {
        match self.generate_expression(expr) {
            Ok(_) => Ok(()),
            Err(e) => Err(Self::codegen_err(e)),
        }
    }

    fn visit_branch(&mut self, branch: &Branch) -> Result<(), Message> {
        match self.generate_branch(branch) {
            Ok(_) => Ok(()),
            Err(e) => Err(Self::codegen_err(e)),
        }
    }

    fn visit_control_flow(&mut self, cf: &ControlFlow) -> Result<(), Message> {
        match self.generate_control_flow(cf) {
            Ok(_) => Ok(()),
            Err(e) => Err(Self::codegen_err(e)),
        }
    }

    fn visit_type_spec(&mut self, type_spec: &TypeSpec) -> Result<(), Message> {
        match self.generate_type_spec(type_spec) {
            Ok(_) => Ok(()),
            Err(e) => Err(Self::codegen_err(e)),
        }
    }

    fn visit_use(&mut self, u: &Use) -> Result<(), Message> {
        match self.generate_use(u) {
            Ok(_) => Ok(()),
            Err(e) => Err(Self::codegen_err(e)),
        }
    }

    fn visit_block(&mut self, block: &Block) -> Result<(), Message> {
        match self.generate_block(block) {
            Ok(_) => Ok(()),
            Err(e) => Err(Self::codegen_err(e)),
        }
    }

    fn visit_value(&mut self, val: &Value) -> Result<(), Message> {
        match self.generate_value(val) {
            Ok(_) => Ok(()),
            Err(e) => Err(Self::codegen_err(e)),
        }
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

    fn codegen_err(err: std::io::Error) -> Message {
        Message {
            location: Location::new(),
            text: format!("Codegen error: {err}"),
        }
    }
}

impl CodeGenStream<'_> {
    fn generate_variable_array_def(&mut self, var_def: &VariableDef) -> Result<(), std::io::Error> {
        let ty = var_def.var_type.get_type();
        let Type::Array { size, value_type } = ty else {
            unreachable!("Called generate_variable_array_def on none array variable");
        };

        let ArraySize::Fixed(size) = size else {
            unreachable!("Array size must be known at this point");
        };

        let type_str = value_type.get_c_type();
        let var_name = var_def.identifier;
        let mutable_str = if var_def.mutability.is_mutable() {
            " "
        } else {
            " const "
        };

        write!(self, "{type_str}{mutable_str}{var_name}[{size}]")?;

        Ok(())
    }

    fn generate_variable_def(&mut self, var_def: &VariableDef) -> Result<(), std::io::Error> {
        if let Type::Array { .. } = var_def.var_type.get_type() {
            return self.generate_variable_array_def(var_def);
        }

        self.generate_type_spec(&var_def.var_type)?;

        write!(
            self,
            "{}{}",
            if var_def.mutability.is_mutable() {
                " "
            } else {
                " const "
            },
            var_def.identifier
        )?;

        Ok(())
    }

    fn generate_expression(&mut self, expr: &Expression) -> Result<(), std::io::Error> {
        let old_mode = self.mode;
        self.mode = CodeGenMode::SourceOnly;

        match &expr.kind {
            ExpressionKind::Unary { operation, node } => {
                match operation {
                    UnaryOperation::RefMut | UnaryOperation::Ref => write!(self, "&")?,
                    UnaryOperation::Not => write!(self, "~")?,
                    UnaryOperation::Deref => write!(self, "*")?,
                };

                self.generate(node)?;
            }
            ExpressionKind::Binary {
                operation: BinaryOperation::Assign,
                lhs,
                rhs,
            } => {
                self.generate(lhs)?;
                write!(self, " = ")?;
                self.generate(rhs)?;
            }
            ExpressionKind::Binary {
                operation,
                lhs,
                rhs,
            } => {
                // write!(self, "(")?;
                self.generate(lhs)?;
                write!(self, " {operation} ")?;
                self.generate(rhs)?;
                // write!(self, ")")?;
            }
            ExpressionKind::Conversion { lhs, ty } => {
                write!(self, "(({})", ty.get_c_type())?;
                self.generate(lhs)?;
                write!(self, ")")?;
            }
            ExpressionKind::Access { lhs, rhs } => {
                self.generate(lhs)?;
                write!(self, ".")?;
                self.generate(rhs)?;
            }
            ExpressionKind::Get { lhs, rhs } => {
                self.generate(lhs)?;
                write!(self, ".")?;
                self.generate(rhs)?;
            }
            ExpressionKind::Indexing { lhs, index } => {
                self.generate(lhs)?;

                write!(self, "[")?;
                self.generate(index)?;
                write!(self, "]")?;
            }
            ExpressionKind::Term { node, .. } => {
                self.generate(node)?;
            }
        }

        self.mode = old_mode;
        Ok(())
    }

    fn generate_type_spec(&mut self, type_spec: &TypeSpec) -> Result<(), std::io::Error> {
        write!(self, "{}", type_spec.get_c_type())
    }

    fn generate_value(&mut self, val: &Value) -> Result<(), std::io::Error> {
        match &val.kind {
            ValueKind::Integer(val) => write!(self, "{val}")?,
            ValueKind::Decimal(val) => write!(self, "{val}")?,
            ValueKind::Identifier(val) => write!(self, "{val}")?,
            ValueKind::Call {
                identifier,
                arguments,
            } => {
                /* at this point, all arguments must be converted to positional */
                write!(self, "{identifier}(")?;

                if !arguments.is_empty() {
                    self.generate_call_param(&arguments[0])?;
                }

                for arg in arguments.iter().skip(1) {
                    write!(self, ", ")?;
                    self.generate_call_param(arg)?;
                }

                write!(self, ")")?;
            }
            ValueKind::Struct {
                identifier,
                components,
            } => {
                // create anonimous variable
                write!(self, "({identifier})")?;

                if components.is_empty() {
                    write!(self, " {{ }}")?;
                } else {
                    let indentation = self.indentation();
                    self.indent += 1;

                    writeln!(self, "\n{indentation}{{")?;
                    for (i, (field_name, field_val)) in components.iter().enumerate() {
                        write!(self, "{indentation}    .{field_name}=")?;
                        self.generate(field_val)?;

                        if i < components.len() {
                            writeln!(self, ",")?;
                        }
                    }

                    self.indent -= 1;
                    write!(self, "{indentation}}}")?;
                }
            }
            ValueKind::Array { components } => {
                write!(self, "{{ ")?;

                for (c_idx, c) in components.iter().enumerate() {
                    self.generate(c)?;

                    if c_idx != components.len() - 1 {
                        write!(self, ", ")?;
                    }
                }

                write!(self, " }}")?;
            }
            ValueKind::Tuple { components } => {
                write!(self, "{{ ")?;

                for (c_idx, c) in components.iter().enumerate() {
                    self.generate(c)?;

                    if c_idx != components.len() - 1 {
                        write!(self, ", ")?;
                    }
                }

                write!(self, " }}")?;
            }
            _ => todo!("Unimplemented for ({:?})", val.kind),
        }

        Ok(())
    }
}

// Call args
impl CodeGenStream<'_> {
    fn generate_call_param(&mut self, arg: &CallArg) -> Result<(), std::io::Error> {
        match &arg.kind {
            CallArgKind::Positional(_, node) => self.generate(node.as_ref()),
            CallArgKind::Notified(..) => {
                unreachable!("Notified CallParam is not allowed in codegen")
            }
        }
    }
}
