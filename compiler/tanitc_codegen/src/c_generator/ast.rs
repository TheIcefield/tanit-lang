use tanitc_ast::{
    expression_utils::{BinaryOperation, UnaryOperation},
    AliasDef, Ast, Block, Branch, BranchKind, CallArg, CallArgKind, ControlFlow, ControlFlowKind,
    EnumDef, Expression, ExpressionKind, ExternDef, FunctionDef, ModuleDef, StructDef, TypeSpec,
    UnionDef, Use, Value, ValueKind, VariableDef, VariantDef, VariantField, Visitor,
};
use tanitc_ident::Ident;
use tanitc_lexer::location::Location;
use tanitc_messages::Message;

use super::{CodeGenMode, CodeGenStream};

use std::{collections::BTreeMap, io::Write};

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

    fn visit_enum_def(&mut self, enum_def: &EnumDef) -> Result<(), Message> {
        match self.generate_enum_def(enum_def) {
            Ok(_) => Ok(()),
            Err(e) => Err(Self::codegen_err(e)),
        }
    }

    fn visit_func_def(&mut self, func_def: &FunctionDef) -> Result<(), Message> {
        match self.generate_func_def(func_def) {
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
    fn codegen_err(err: std::io::Error) -> Message {
        Message {
            location: Location::new(),
            text: format!("Codegen error: {err}"),
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
            Ast::EnumDef(node) => self.generate_enum_def(node),
            Ast::FuncDef(node) => self.generate_func_def(node),
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

    fn generate_module_def(&mut self, module_def: &ModuleDef) -> Result<(), std::io::Error> {
        if module_def.body.is_none() {
            self.generate_external_module(module_def)
        } else {
            self.generate_internal_module(module_def)
        }
    }

    fn generate_struct_def(&mut self, struct_def: &StructDef) -> Result<(), std::io::Error> {
        let old_mode = self.mode;
        self.mode = CodeGenMode::HeaderOnly;

        writeln!(self, "typedef struct {{")?;
        for (field_id, field_info) in struct_def.fields.iter() {
            self.generate_type_spec(&field_info.ty)?;
            write!(self, " {}", field_id)?;
            writeln!(self, ";")?;
        }
        write!(self, "}} {}", struct_def.identifier)?;

        writeln!(self, ";")?;

        self.mode = old_mode;
        Ok(())
    }

    fn generate_union_def(&mut self, union_def: &UnionDef) -> Result<(), std::io::Error> {
        let old_mode = self.mode;
        self.mode = CodeGenMode::HeaderOnly;

        writeln!(self, "typedef union {{")?;
        for (field_id, field_info) in union_def.fields.iter() {
            self.generate_type_spec(&field_info.ty)?;
            write!(self, " {}", field_id)?;
            writeln!(self, ";")?;
        }
        write!(self, "}} {}", union_def.identifier)?;

        writeln!(self, ";")?;

        self.mode = old_mode;
        Ok(())
    }

    fn generate_variant_def(&mut self, variant_def: &VariantDef) -> Result<(), std::io::Error> {
        let old_mode = self.mode;
        self.mode = CodeGenMode::HeaderOnly;

        writeln!(self, "typedef struct {{")?;

        self.generate_variant_enum(variant_def.identifier, &variant_def.fields)?;
        self.generate_variant_data(variant_def.identifier, &variant_def.fields)?;

        write!(self, "}} {}", variant_def.identifier)?;

        writeln!(self, ";")?;

        self.mode = old_mode;
        Ok(())
    }

    fn generate_enum_def(&mut self, enum_def: &EnumDef) -> Result<(), std::io::Error> {
        let old_mode = self.mode;
        self.mode = CodeGenMode::HeaderOnly;

        writeln!(self, "typedef enum {{")?;

        for field in enum_def.fields.iter() {
            writeln!(self, "    {} = {},", field.0, field.1.unwrap_or_default())?;
        }

        writeln!(self, "}} {};", enum_def.identifier)?;

        self.mode = old_mode;

        Ok(())
    }

    fn generate_func_def(&mut self, func_def: &FunctionDef) -> Result<(), std::io::Error> {
        let old_mode = self.mode;
        self.mode = if func_def.body.is_some() {
            CodeGenMode::Both
        } else {
            CodeGenMode::HeaderOnly
        };

        self.generate_type_spec(&func_def.return_type)?;

        write!(self, " {}", func_def.identifier)?;

        // generate parameters
        write!(self, "(")?;
        if !func_def.parameters.is_empty() {
            self.generate(&func_def.parameters[0])?;
        }

        for param in func_def.parameters.iter().skip(1) {
            write!(self, ", ")?;
            self.generate(param)?;
        }
        write!(self, ")")?;

        self.mode = CodeGenMode::HeaderOnly;
        writeln!(self, ";")?;

        if let Some(body) = &func_def.body {
            self.mode = CodeGenMode::SourceOnly;
            self.generate(body)?;
        }

        self.mode = old_mode;
        Ok(())
    }

    fn generate_variable_def(&mut self, var_def: &VariableDef) -> Result<(), std::io::Error> {
        self.generate_type_spec(&var_def.var_type)?;

        write!(
            self,
            "{}{}",
            if var_def.is_mutable { " " } else { " const " },
            var_def.identifier
        )?;

        Ok(())
    }

    fn generate_alias_def(&mut self, alias_def: &AliasDef) -> Result<(), std::io::Error> {
        let old_mode = self.mode;
        self.mode = CodeGenMode::HeaderOnly;

        write!(
            self,
            "typedef {} {}",
            alias_def.value.get_c_type(),
            alias_def.identifier
        )?;

        writeln!(self, ";")?;

        self.mode = old_mode;

        Ok(())
    }

    fn generate_expression(&mut self, expr: &Expression) -> Result<(), std::io::Error> {
        let old_mode = self.mode;
        self.mode = CodeGenMode::SourceOnly;

        match &expr.kind {
            ExpressionKind::Unary { operation, node } => {
                let keep_space = if *operation == UnaryOperation::RefMut {
                    " "
                } else {
                    ""
                };
                write!(self, "{}{}", operation, keep_space)?;
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
                write!(self, " {} ", operation)?;
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
            ExpressionKind::Term { node, .. } => {
                self.generate(node)?;
            }
        }

        self.mode = old_mode;
        Ok(())
    }

    fn generate_branch(&mut self, branch: &Branch) -> Result<(), std::io::Error> {
        let old_mode = self.mode;
        self.mode = CodeGenMode::SourceOnly;

        match &branch.kind {
            BranchKind::Loop { body } => {
                write!(self, "while (1)")?;

                self.generate(body)?;
            }
            BranchKind::While { body, condition } => {
                write!(self, "while (")?;

                self.generate(condition)?;

                writeln!(self, ")")?;

                self.generate(body)?;
            }
            BranchKind::If { condition, body } => {
                write!(self, "if (")?;
                self.generate(condition)?;
                writeln!(self, ")")?;

                self.generate(body)?;
            }
            BranchKind::Else { body } => {
                writeln!(self, "else")?;
                self.generate(body)?;
            }
        }
        self.mode = old_mode;
        Ok(())
    }

    fn generate_control_flow(&mut self, cf: &ControlFlow) -> Result<(), std::io::Error> {
        let old_mode = self.mode;
        self.mode = CodeGenMode::SourceOnly;

        write!(self, "return ")?;

        if let ControlFlowKind::Return { ret } = &cf.kind {
            if let Some(expr) = ret.as_ref() {
                self.generate(expr)?;
            }
        }

        self.mode = old_mode;
        Ok(())
    }

    fn generate_type_spec(&mut self, type_spec: &TypeSpec) -> Result<(), std::io::Error> {
        write!(self, "{}", type_spec.get_c_type())
    }

    fn generate_use(&mut self, _u: &Use) -> Result<(), std::io::Error> {
        Ok(())
    }

    fn generate_block(&mut self, block: &Block) -> Result<(), std::io::Error> {
        if !block.is_global {
            writeln!(self, "{{")?;
        }

        for stmt in block.statements.iter() {
            self.generate(stmt)?;

            match stmt {
                Ast::Expression(_)
                | Ast::ControlFlow(_)
                | Ast::VariableDef(_)
                | Ast::Value(Value {
                    kind: ValueKind::Call { .. },
                    ..
                }) => write!(self, ";")?,
                _ => {}
            }

            writeln!(self)?;
        }

        if !block.is_global {
            writeln!(self, "}}")?;
        }
        Ok(())
    }

    fn generate_value(&mut self, val: &Value) -> Result<(), std::io::Error> {
        match &val.kind {
            ValueKind::Integer(val) => write!(self, "{}", *val)?,
            ValueKind::Decimal(val) => write!(self, "{}", *val)?,
            ValueKind::Identifier(val) => write!(self, "{}", val)?,
            ValueKind::Call {
                identifier,
                arguments,
            } => {
                /* at this point, all arguments must be converted to positional */
                write!(self, "{}(", identifier)?;

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
                writeln!(self, "({identifier}){{")?;

                for (i, (field_name, field_val)) in components.iter().enumerate() {
                    write!(self, ".{field_name}=")?;
                    self.generate(field_val)?;

                    if i < components.len() {
                        writeln!(self, ",")?;
                    }
                }

                write!(self, "}}")?;
            }
            _ => todo!("Unimplemented for ({:?})", val.kind),
        }

        Ok(())
    }
}

impl CodeGenStream<'_> {
    fn generate_internal_module(&mut self, module_def: &ModuleDef) -> std::io::Result<()> {
        if let Some(body) = &module_def.body {
            self.generate_block(body)?;
        }

        Ok(())
    }

    fn generate_external_module(&mut self, module_def: &ModuleDef) -> std::io::Result<()> {
        let old_mode = self.mode;
        self.mode = CodeGenMode::HeaderOnly;

        writeln!(self, "#include \"./{}.tt.h\"", module_def.identifier)?;

        self.mode = old_mode;

        Ok(())
    }
}

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

impl CodeGenStream<'_> {
    fn generate_variant_enum(
        &mut self,
        variant_id: Ident,
        fields: &BTreeMap<Ident, VariantField>,
    ) -> Result<(), std::io::Error> {
        let enum_id = tanitc_ast::variant_utils::get_variant_data_kind_id(variant_id);
        let field_id = Ident::from("__kind__".to_string());

        writeln!(self, "typedef enum {enum_id} {{")?;

        for (field_id, _) in fields.iter() {
            writeln!(self, "    {field_id},")?;
        }

        writeln!(self, "}} {field_id};")?;

        Ok(())
    }

    fn generate_variant_common_field(
        &mut self,
        variant_id: Ident,
        field_id: Ident,
    ) -> Result<(), std::io::Error> {
        writeln!(self, "typedef struct __{variant_id}__{field_id}__ {{")?;
        writeln!(self, "}} __{variant_id}__{field_id}__ {field_id}")?;
        Ok(())
    }

    fn generate_variant_struct_field(
        &mut self,
        variant_id: Ident,
        field_id: Ident,
        subfields: &BTreeMap<Ident, TypeSpec>,
    ) -> Result<(), std::io::Error> {
        writeln!(self, "typedef struct __{variant_id}__{field_id}__ {{")?;
        for (subfield_id, subfield_type) in subfields.iter() {
            let subfield_type = subfield_type.get_type();
            writeln!(self, "    {subfield_type} {subfield_id};")?;
        }
        writeln!(self, "}} __{variant_id}__{field_id}__ {field_id}")?;

        Ok(())
    }

    fn generate_variant_tuple_field(
        &mut self,
        variant_id: Ident,
        field_id: Ident,
        components: &[TypeSpec],
    ) -> Result<(), std::io::Error> {
        writeln!(self, "typedef struct __{variant_id}__{field_id}__ {{")?;
        for (field_num, field_type) in components.iter().enumerate() {
            let field_type = field_type.get_type();
            writeln!(self, "    {field_type} _{field_num};")?;
        }
        writeln!(self, "}} __{variant_id}__{field_id}__ {field_id}")?;

        Ok(())
    }

    fn generate_variant_data(
        &mut self,
        variant_id: Ident,
        fields: &BTreeMap<Ident, VariantField>,
    ) -> Result<(), std::io::Error> {
        let union_id = tanitc_ast::variant_utils::get_variant_data_type_id(variant_id);
        let field_id = Ident::from("__data__".to_string());

        writeln!(self, "typedef union {union_id} {{")?;

        for (field_id, field_data) in fields.iter() {
            match field_data {
                VariantField::Common => {
                    self.generate_variant_common_field(variant_id, *field_id)?
                }
                VariantField::StructLike(subfields) => {
                    self.generate_variant_struct_field(variant_id, *field_id, subfields)?
                }
                VariantField::TupleLike(components) => {
                    self.generate_variant_tuple_field(variant_id, *field_id, components)?
                }
            }
        }

        writeln!(self, "}} {field_id};")?;

        Ok(())
    }
}

impl CodeGenStream<'_> {
    fn generate_extern_def(&mut self, extern_def: &ExternDef) -> Result<(), std::io::Error> {
        let mode = self.mode;
        self.mode = CodeGenMode::HeaderOnly;

        for func_def in extern_def.functions.iter() {
            self.generate_func_def(func_def)?;
        }

        self.mode = mode;

        Ok(())
    }
}
