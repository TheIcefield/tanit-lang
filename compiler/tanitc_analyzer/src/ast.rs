use crate::{scope::ScopeUnitKind, symbol::Symbol};

use super::{scope::ScopeUnit, symbol::SymbolData, Analyzer};

use tanitc_ast::{
    self,
    attributes::Safety,
    expression_utils::{BinaryOperation, UnaryOperation},
    variant_utils, AliasDef, Ast, Block, Branch, BranchKind, CallArg, CallArgKind, ControlFlow,
    ControlFlowKind, EnumDef, Expression, ExpressionKind, FunctionDef, ModuleDef, StructDef,
    TypeSpec, UnionDef, Use, Value, ValueKind, VariableDef, VariantDef, VariantField, VisitorMut,
};
use tanitc_ident::Ident;
use tanitc_lexer::location::Location;
use tanitc_messages::Message;
use tanitc_ty::Type;

use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashMap},
};

impl VisitorMut for Analyzer {
    fn visit_module_def(&mut self, module_def: &mut ModuleDef) -> Result<(), Message> {
        if self.has_symbol(module_def.identifier) {
            return Err(Message::multiple_ids(
                module_def.location,
                module_def.identifier,
            ));
        }

        self.add_symbol(self.create_symbol(module_def.identifier, SymbolData::ModuleDef));

        let safety = self.get_current_safety();

        self.scope.push(ScopeUnit {
            kind: ScopeUnitKind::Module(module_def.identifier),
            safety: module_def.attrs.safety.unwrap_or(safety),
        });

        if let Some(body) = &mut module_def.body {
            self.visit_block(body)?;
        }

        self.scope.pop();

        Ok(())
    }

    fn visit_struct_def(&mut self, struct_def: &mut StructDef) -> Result<(), Message> {
        if self.has_symbol(struct_def.identifier) {
            return Err(Message::multiple_ids(
                struct_def.location,
                struct_def.identifier,
            ));
        }

        self.scope.push(ScopeUnit {
            kind: ScopeUnitKind::Struct(struct_def.identifier),
            safety: self.get_current_safety(),
        });
        for internal in struct_def.internals.iter_mut() {
            internal.accept_mut(self)?;
        }

        for (field_id, field_ty) in struct_def.fields.iter() {
            self.add_symbol(self.create_symbol(
                *field_id,
                SymbolData::StructField {
                    struct_id: struct_def.identifier,
                    ty: field_ty.get_type(),
                },
            ));
        }

        self.scope.pop();

        self.add_symbol(self.create_symbol(struct_def.identifier, SymbolData::StructDef));

        Ok(())
    }

    fn visit_union_def(&mut self, union_def: &mut UnionDef) -> Result<(), Message> {
        if self.has_symbol(union_def.identifier) {
            return Err(Message::multiple_ids(
                union_def.location,
                union_def.identifier,
            ));
        }

        self.scope.push(ScopeUnit {
            kind: ScopeUnitKind::Union(union_def.identifier),
            safety: self.get_current_safety(),
        });
        for internal in union_def.internals.iter_mut() {
            internal.accept_mut(self)?;
        }

        for (field_id, field_ty) in union_def.fields.iter() {
            self.add_symbol(self.create_symbol(
                *field_id,
                SymbolData::UnionField {
                    union_id: union_def.identifier,
                    ty: field_ty.get_type(),
                },
            ));
        }

        self.scope.pop();

        self.add_symbol(self.create_symbol(union_def.identifier, SymbolData::UnionDef));

        Ok(())
    }

    fn visit_variant_def(&mut self, variant_def: &mut VariantDef) -> Result<(), Message> {
        // TODO: variants #8
        if self.compile_options.allow_variants {
            self.analyze_variant_def(variant_def)
        } else {
            Err(Message::new(
                variant_def.location,
                "Variants not supported in 0.1.0",
            ))
        }
    }

    fn visit_enum_def(&mut self, enum_def: &mut EnumDef) -> Result<(), Message> {
        if self.has_symbol(enum_def.identifier) {
            return Err(Message::multiple_ids(
                enum_def.location,
                enum_def.identifier,
            ));
        }

        let mut counter = 0usize;
        let mut components = Vec::<(Ident, usize)>::new();
        for field in enum_def.fields.iter_mut() {
            if let Some(value) = field.1 {
                counter = *value;
            }

            // mark unmarked enum fields
            *field.1 = Some(counter);

            components.push((*field.0, counter));

            counter += 1;
        }

        self.add_symbol(self.create_symbol(enum_def.identifier, SymbolData::EnumDef));

        self.scope.push(ScopeUnit {
            kind: ScopeUnitKind::Enum(enum_def.identifier),
            safety: self.get_current_safety(),
        });

        for comp in components.iter() {
            self.add_symbol(self.create_symbol(
                comp.0,
                SymbolData::EnumComponent {
                    enum_id: enum_def.identifier,
                    val: comp.1,
                },
            ));
        }
        self.scope.pop();

        Ok(())
    }

    fn visit_func_def(&mut self, func_def: &mut FunctionDef) -> Result<(), Message> {
        if self.has_symbol(func_def.identifier) {
            return Err(Message::multiple_ids(
                func_def.location,
                func_def.identifier,
            ));
        }

        let safety = self.get_current_safety();

        self.scope.push(ScopeUnit {
            kind: ScopeUnitKind::Func(func_def.identifier),
            safety,
        });

        let mut parameters = Vec::<(Ident, Type)>::new();
        for p in func_def.parameters.iter_mut() {
            if let Ast::VariableDef(node) = p {
                parameters.push((node.identifier, node.var_type.get_type()));
                p.accept_mut(self)?;
            }
        }

        self.scope.pop();

        self.add_symbol(self.create_symbol(
            func_def.identifier,
            SymbolData::FunctionDef {
                parameters,
                return_type: func_def.return_type.get_type(),
                is_declaration: func_def.body.is_some(),
            },
        ));

        self.scope.push(ScopeUnit {
            kind: ScopeUnitKind::Func(func_def.identifier),
            safety,
        });

        if let Some(body) = &mut func_def.body {
            body.accept_mut(self)?;
        }

        self.scope.pop();

        Ok(())
    }

    fn visit_variable_def(&mut self, var_def: &mut VariableDef) -> Result<(), Message> {
        if self.has_symbol(var_def.identifier) {
            return Err(Message::multiple_ids(var_def.location, var_def.identifier));
        }

        self.add_symbol(self.create_symbol(
            var_def.identifier,
            SymbolData::VariableDef {
                var_type: var_def.var_type.get_type(),
                is_mutable: var_def.is_mutable,
                is_initialization: false,
            },
        ));

        Ok(())
    }

    fn visit_alias_def(&mut self, alias_def: &mut AliasDef) -> Result<(), Message> {
        if self.has_symbol(alias_def.identifier) {
            return Err(Message::multiple_ids(
                alias_def.location,
                alias_def.identifier,
            ));
        }

        self.add_symbol(self.create_symbol(
            alias_def.identifier,
            SymbolData::AliasDef {
                ty: alias_def.value.get_type(),
            },
        ));

        Ok(())
    }

    fn visit_expression(&mut self, expr: &mut Expression) -> Result<(), Message> {
        let ret = match &mut expr.kind {
            ExpressionKind::Binary {
                operation,
                lhs,
                rhs,
            } => self.analyze_binary_expr(operation, lhs.as_mut(), rhs.as_mut()),
            ExpressionKind::Unary { operation, node } => self.analyze_unary_expr(operation, node),
            ExpressionKind::Access { lhs, rhs } => self.analyze_access_expr(lhs, rhs),
            ExpressionKind::Conversion { .. } => self.analyze_conversion_expr(),
            ExpressionKind::Term { node, .. } => self.analyze_term_expr(node),
        };

        match ret {
            Ok(Some(processed_node)) => *expr = processed_node,
            Err(mut msg) => {
                msg.location = expr.location;
                return Err(msg);
            }
            _ => {}
        }

        Ok(())
    }

    fn visit_branch(&mut self, branch: &mut Branch) -> Result<(), Message> {
        let safety = self.get_current_safety();

        let analyze_body = |body: &mut Ast, analyzer: &mut Analyzer| -> Result<(), Message> {
            if let Ast::Block(node) = body {
                for stmt in node.statements.iter_mut() {
                    stmt.accept_mut(analyzer)?;
                }
            }

            Ok(())
        };

        let analyze_condition =
            |condition: &mut Ast, analyzer: &mut Analyzer| -> Result<(), Message> {
                if let Ast::Expression(node) = condition {
                    analyzer.visit_expression(node)?;
                }

                Ok(())
            };

        match &mut branch.kind {
            BranchKind::While { body, condition } => {
                let cnt = self.counter();
                self.scope.push(ScopeUnit {
                    kind: ScopeUnitKind::Loop(cnt),
                    safety,
                });

                condition.accept_mut(self)?;

                analyze_condition(condition.as_mut(), self)?;
                analyze_body(body.as_mut(), self)?;

                self.scope.pop();

                Ok(())
            }
            BranchKind::Loop { body } => {
                let cnt = self.counter();
                self.scope.push(ScopeUnit {
                    kind: ScopeUnitKind::Loop(cnt),
                    safety,
                });

                analyze_body(body.as_mut(), self)?;

                self.scope.pop();

                Ok(())
            }
            BranchKind::If { body, condition } => {
                analyze_condition(condition.as_mut(), self)?;
                analyze_body(body.as_mut(), self)?;

                Ok(())
            }
            BranchKind::Else { body } => {
                analyze_body(body.as_mut(), self)?;

                Ok(())
            }
        }
    }

    fn visit_control_flow(&mut self, cf: &mut ControlFlow) -> Result<(), Message> {
        let is_in_func = {
            let mut flag = false;
            for s in self.scope.iter().rev() {
                if matches!(s.kind, ScopeUnitKind::Func(_)) {
                    flag = true;
                    break;
                }
            }

            flag
        };

        let is_in_loop = {
            let mut flag = false;
            for s in self.scope.iter().rev() {
                if matches!(s.kind, ScopeUnitKind::Loop(_)) {
                    flag = true;
                    break;
                }
            }

            flag
        };

        match &mut cf.kind {
            ControlFlowKind::Break { ret } | ControlFlowKind::Return { ret } => {
                if let Some(expr) = ret {
                    expr.accept_mut(self)?;
                }
            }
            _ => {}
        }

        let is_ret = matches!(cf.kind, ControlFlowKind::Return { .. });

        if (!is_ret && !is_in_loop) || (is_ret && !is_in_func) {
            return Err(Message::new(
                cf.location,
                &format!("Unexpected {} statement", cf.kind.to_str()),
            ));
        }

        Ok(())
    }

    fn visit_type_spec(&mut self, _type_spec: &mut TypeSpec) -> Result<(), Message> {
        // if self.has_type(type_spec.get_type()) {
        //     return Err(Message::undefined_type(
        //         type_spec.location,
        //         type_spec..get_type(),
        //     ));
        // }
        Ok(())
    }

    fn visit_use(&mut self, _u: &mut Use) -> Result<(), Message> {
        Ok(())
    }

    fn visit_block(&mut self, block: &mut Block) -> Result<(), Message> {
        if block.is_global {
            self.analyze_global_block(block)?;
        } else {
            self.analyze_local_block(block)?;
        }

        Ok(())
    }

    fn visit_value(&mut self, val: &mut Value) -> Result<(), Message> {
        match &mut val.kind {
            ValueKind::Integer(_) => Ok(()),

            ValueKind::Decimal(_) => Ok(()),

            ValueKind::Text(_) => Ok(()),

            ValueKind::Identifier(id) => {
                if self.has_symbol(*id) {
                    Ok(())
                } else {
                    Err(Message::undefined_id(val.location, *id))
                }
            }

            ValueKind::Call {
                identifier: func_name,
                arguments: call_args,
            } => {
                let Some(func_symbol) = self.get_first_symbol(*func_name) else {
                    return Err(Message::undefined_id(val.location, *func_name));
                };

                self.analyze_call(&func_symbol, call_args, val.location)?;

                Ok(())
            }

            ValueKind::Struct { .. } => self.analyze_struct_value(val),

            ValueKind::Tuple { components } => {
                for comp in components.iter_mut() {
                    comp.accept_mut(self)?;
                }

                Ok(())
            }

            ValueKind::Array { components } => {
                if components.is_empty() {
                    return Ok(());
                }

                let comp_type = self.get_type(&components[0]);

                for comp in components.iter().enumerate() {
                    let current_comp_type = self.get_type(comp.1);
                    if comp_type != current_comp_type {
                        let comp_index = comp.0 + 1;
                        let suffix = get_ordinal_number_suffix(comp.0);
                        return Err(Message::from_string(
                            val.location,
                            format!(
                                "Array type is declared like {comp_type}, but {comp_index}{suffix} element has type {current_comp_type}",
                            ),
                        ));
                    }
                }

                Ok(())
            }
        }
    }
}

// Type
impl Analyzer {
    fn get_type(&self, node: &Ast) -> Type {
        match node {
            Ast::AliasDef(node) => self.get_alias_def_type(node),
            Ast::VariableDef(node) => self.get_var_def_type(node),
            Ast::Expression(node) => self.get_expr_type(node),
            Ast::Value(node) => self.get_value_type(node),
            _ => todo!("GetType: {}", node.name()),
        }
    }

    fn get_alias_def_type(&self, alias_def: &AliasDef) -> Type {
        alias_def.value.get_type()
    }

    fn get_var_def_type(&self, var_def: &VariableDef) -> Type {
        var_def.var_type.get_type()
    }

    fn get_expr_type(&self, expr: &Expression) -> Type {
        match &expr.kind {
            ExpressionKind::Binary {
                operation,
                lhs,
                rhs,
            } => match operation {
                BinaryOperation::LogicalNe
                | BinaryOperation::LogicalEq
                | BinaryOperation::LogicalLt
                | BinaryOperation::LogicalLe
                | BinaryOperation::LogicalGt
                | BinaryOperation::LogicalGe => Type::Bool,

                _ => {
                    let lhs_type = self.get_type(lhs);

                    if let Type::Auto = lhs_type {
                        return self.get_type(rhs);
                    }

                    lhs_type
                }
            },
            ExpressionKind::Unary { operation, node } => {
                let node_type = self.get_type(node);

                let (is_ref, is_mutable) = if *operation == UnaryOperation::Ref {
                    (true, false)
                } else if *operation == UnaryOperation::RefMut {
                    (true, true)
                } else {
                    (false, false)
                };

                if is_ref {
                    return Type::Ref {
                        ref_to: Box::new(node_type),
                        is_mutable,
                    };
                }

                node_type
            }
            ExpressionKind::Conversion { ty, .. } => ty.get_type(),
            ExpressionKind::Access { rhs, .. } => self.get_type(rhs),
            ExpressionKind::Term { ty, .. } => ty.clone(),
        }
    }

    fn get_value_type(&self, val: &Value) -> Type {
        match &val.kind {
            ValueKind::Text(_) => Type::Ref {
                ref_to: Box::new(Type::Str),
                is_mutable: false,
            },
            ValueKind::Decimal(_) => Type::F32,
            ValueKind::Integer(_) => Type::I32,
            ValueKind::Identifier(id) => {
                if let Some(ss) = self.get_symbols(id) {
                    for s in ss.iter().rev() {
                        if self.scope.0.starts_with(&s.scope.0) {
                            if let SymbolData::VariableDef { var_type, .. } = &s.data {
                                return var_type.clone();
                            }
                        }
                    }
                }
                Type::new()
            }
            ValueKind::Struct { identifier, .. } => Type::Custom(identifier.to_string()),
            ValueKind::Tuple { components } => {
                let mut comp_vec = Vec::<Type>::new();
                for comp in components.iter() {
                    comp_vec.push(self.get_type(comp));
                }
                Type::Tuple(comp_vec)
            }
            ValueKind::Array { components } => {
                let len = components.len();
                if len == 0 {
                    return Type::Array {
                        size: None,
                        value_type: Box::new(Type::Auto),
                    };
                }

                Type::Array {
                    size: None,
                    value_type: Box::new(self.get_type(&components[0])),
                }
            }
            ValueKind::Call { identifier, .. } => {
                if let Some(ss) = self.get_first_symbol(*identifier) {
                    if let SymbolData::FunctionDef { return_type, .. } = &ss.data {
                        return return_type.clone();
                    }
                }
                Type::new()
            }
        }
    }
}

// Call
impl Analyzer {
    fn check_arg_count(
        &self,
        func_name: Ident,
        arguments: &[CallArg],
        parameters: &[(Ident, Type)],
        location: Location,
    ) -> Result<(), Message> {
        let expected_len = parameters.len();
        let actual_len = arguments.len();

        let many_or_few = match actual_len.cmp(&expected_len) {
            Ordering::Greater => "many",
            Ordering::Less => "few",
            Ordering::Equal => "",
        };

        if actual_len != expected_len {
            return Err(Message::new(
                location,
                &format!(
                    "Too {many_or_few} arguments passed in function \"{func_name}\", expected: {expected_len}, actually: {actual_len}",
                ),
            ));
        }

        Ok(())
    }

    fn analyze_positional_arg(
        &self,
        func_name: Ident,
        func_params: &[(Ident, Type)],
        arg_idx: usize,
        arg_value: &Ast,
        positional_skipped: &mut bool,
    ) -> Result<usize, Message> {
        if *positional_skipped {
            return Err(Message::from_string(
                arg_value.location(),
                format!("In function \"{func_name}\" call: positional parameter \"{arg_idx}\" must be passed before notified",
            )));
        }

        let func_param_type = &func_params[arg_idx].1;
        let expr_type = self.get_type(arg_value);

        if expr_type != *func_param_type {
            return Err(Message::from_string(
                arg_value.location(),
                format!("Mismatched types. In function \"{func_name}\" call: positional parameter \"{arg_idx}\" has type \"{expr_type}\" but expected \"{func_param_type}\""),
            ));
        }

        Ok(arg_idx)
    }

    fn analyze_notified_arg(
        &self,
        func_name: Ident,
        func_params: &[(Ident, Type)],
        arg_id: Ident,
        arg_value: &Ast,
        positional_skipped: &mut bool,
    ) -> Result<usize, Message> {
        *positional_skipped = true;
        let location = arg_value.location();

        // check if such parameter declared in the function
        for (param_index, (param_name, param_type)) in func_params.iter().enumerate() {
            if *param_name == arg_id {
                let arg_type = self.get_type(arg_value);
                if *param_type != arg_type {
                    return Err(Message::from_string(
                        location,
                        format!("Mismatched types. In function \"{func_name}\" call: notified parameter \"{arg_id}\" has type \"{arg_type}\" but expected \"{param_type}\""),
                    ));
                }

                return Ok(param_index);
            }
        }

        Err(Message::from_string(
            location,
            format!("No parameter named \"{arg_id}\" in function \"{func_name}\""),
        ))
    }

    fn analyze_arg(
        &mut self,
        func_name: Ident,
        func_params: &[(Ident, Type)],
        arg: &mut CallArg,
        positional_skipped: &mut bool,
    ) -> Result<(), Message> {
        let res = match &arg.kind {
            CallArgKind::Notified(arg_id, arg_value) => self.analyze_notified_arg(
                func_name,
                func_params,
                *arg_id,
                arg_value,
                positional_skipped,
            ),
            CallArgKind::Positional(arg_idx, arg_value) => self.analyze_positional_arg(
                func_name,
                func_params,
                *arg_idx,
                arg_value,
                positional_skipped,
            ),
        };

        match res {
            Ok(arg_position) => {
                let arg_value = match &mut arg.kind {
                    CallArgKind::Notified(_, arg_value) => std::mem::take(arg_value),
                    CallArgKind::Positional(_, arg_value) => std::mem::take(arg_value),
                };

                arg.kind = CallArgKind::Positional(arg_position, arg_value);
            }
            Err(err) => self.error(err),
        }

        Ok(())
    }

    fn analyze_call(
        &mut self,
        func_symbol: &Symbol,
        call_args: &mut [CallArg],
        location: Location,
    ) -> Result<(), Message> {
        let func_name = func_symbol.id;

        let SymbolData::FunctionDef {
            parameters: func_params,
            ..
        } = &func_symbol.data
        else {
            return Err(Message::undefined_func(location, func_name));
        };

        if func_name.is_built_in() {
            return Ok(());
        }

        self.check_arg_count(func_name, call_args, func_params, location)?;

        let mut positional_skipped = false;
        for call_arg in call_args.iter_mut() {
            if let Err(err) =
                self.analyze_arg(func_name, func_params, call_arg, &mut positional_skipped)
            {
                self.error(err);
            }
        }

        Ok(())
    }
}

// Alias
impl Analyzer {
    fn find_alias_value(&self, alias_type: &Type) -> Option<Type> {
        if let Type::Custom(id) = alias_type {
            let type_id = Ident::from(id.clone());
            let mut ss = self.table.get_symbols();
            ss.retain(|s| s.id == type_id && matches!(s.data, SymbolData::AliasDef { .. }));

            if ss.len() == 1 {
                if let SymbolData::AliasDef { ty } = &ss[0].data {
                    if let Some(alias_to) = self.find_alias_value(ty) {
                        Some(alias_to)
                    } else {
                        Some(ty.clone())
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

// Expression
impl Analyzer {
    fn analyze_unary_expr(
        &mut self,
        operation: &UnaryOperation,
        node: &mut Box<Ast>,
    ) -> Result<Option<Expression>, Message> {
        node.accept_mut(self)?;

        let does_mutate = *operation == UnaryOperation::RefMut;

        if let Ast::Value(Value {
            location,
            kind: ValueKind::Identifier(id),
        }) = node.as_ref()
        {
            let Some(ss) = self.get_symbols(id) else {
                return Err(Message::undefined_id(*location, *id));
            };

            let first = &ss[0];

            if let SymbolData::VariableDef { is_mutable, .. } = &first.data {
                if !*is_mutable && does_mutate {
                    return Err(Message {
                        location: *location,
                        text: format!("Mutable reference to immutable variable \"{id}\""),
                    });
                }
            }
        }

        Ok(None)
    }

    fn analyze_binary_expr(
        &mut self,
        operation: &BinaryOperation,
        lhs: &mut Ast,
        rhs: &mut Ast,
    ) -> Result<Option<Expression>, Message> {
        rhs.accept_mut(self)?;
        let rhs_type = self.get_type(rhs);

        let does_mutate = *operation == BinaryOperation::Assign
            || *operation == BinaryOperation::SubAssign
            || *operation == BinaryOperation::AddAssign
            || *operation == BinaryOperation::DivAssign
            || *operation == BinaryOperation::ModAssign
            || *operation == BinaryOperation::MulAssign
            || *operation == BinaryOperation::BitwiseAndAssign
            || *operation == BinaryOperation::BitwiseOrAssign
            || *operation == BinaryOperation::BitwiseXorAssign
            || *operation == BinaryOperation::BitwiseShiftLAssign
            || *operation == BinaryOperation::BitwiseShiftRAssign;

        if let Ast::VariableDef(node) = lhs {
            if self.has_symbol(node.identifier) {
                return Err(Message::multiple_ids(rhs.location(), node.identifier));
            }

            if Type::Auto == node.var_type.get_type() {
                node.var_type.ty = rhs_type.clone();
            }

            let var_type = node.var_type.get_type();

            let mut alias_to = self.find_alias_value(&var_type);

            if var_type == rhs_type {
                alias_to = None;
            }

            if alias_to.is_none() && var_type != rhs_type {
                return Err(Message {
                    location: node.location,
                    text: format!(
                        "Cannot perform operation on objects with different types: {var_type:?} and {rhs_type:?}",
                    ),
                });
            } else if alias_to.as_ref().is_some_and(|ty| rhs_type != *ty) {
                return Err(Message {
                    location: node.location,
                    text: format!(
                        "Cannot perform operation on objects with different types: {var_type:?} (aka: {}) and {rhs_type:?}",
                        alias_to.unwrap()
                    ),
                });
            }

            self.add_symbol(self.create_symbol(
                node.identifier,
                SymbolData::VariableDef {
                    var_type: node.var_type.get_type(),
                    is_mutable: node.is_mutable,
                    is_initialization: true,
                },
            ));
        } else if let Ast::Value(node) = lhs {
            match &node.kind {
                ValueKind::Identifier(id) => {
                    if let Some(s) = self.get_first_symbol(*id) {
                        if let SymbolData::VariableDef {
                            is_mutable,
                            var_type,
                            ..
                        } = &s.data
                        {
                            if let Type::Ref { is_mutable, .. } = var_type {
                                if !*is_mutable && does_mutate {
                                    self.error(Message::new(
                                        Location::new(),
                                        &format!(
                                            "Reference \"{id}\" is immutable in current scope",
                                        ),
                                    ));
                                }
                            } else if !*is_mutable && does_mutate {
                                self.error(Message::new(
                                    Location::new(),
                                    &format!("Variable \"{id}\" is immutable in current scope",),
                                ));
                            }
                        }
                    }
                }
                ValueKind::Integer(..) | ValueKind::Decimal(..) => {}
                ValueKind::Text(..) => self.error(Message::new(
                    Location::new(),
                    "Cannot perform operation with text in this context",
                )),
                ValueKind::Array { .. } => self.error(Message::new(
                    Location::new(),
                    "Cannot perform operation with array in this context",
                )),
                ValueKind::Tuple { .. } => self.error(Message::new(
                    Location::new(),
                    "Cannot perform operation with tuple in this context",
                )),
                _ => self.error(Message::new(
                    Location::new(),
                    "Cannot perform operation with this object",
                )),
            }
        } else {
            lhs.accept_mut(self)?;
            let lhs_type = self.get_type(lhs);

            if lhs_type != rhs_type {
                self.error(Message::new(
                    rhs.location(),
                    &format!(
                        "Cannot perform operation on objects with different types: {:?} and {:?}",
                        lhs_type, rhs_type
                    ),
                ));
            }
        }

        Ok(None)
    }

    fn analyze_access_expr(
        &mut self,
        lhs: &mut Ast,
        rhs: &mut Ast,
    ) -> Result<Option<Expression>, Message> {
        let mut ids = Vec::<Ident>::new();

        let mut variant_ident: Option<Ident> = None;

        Self::preprocess_access_tree(&mut ids, lhs, rhs)?;

        let scope = self.scope.clone();
        let s = {
            let ss = self.table.access_symbol(&ids, &scope);

            if ss.is_empty() {
                return Err(Message::undefined_id(lhs.location(), *ids.last().unwrap()));
            }

            let s = ss[0].clone();

            if s.scope.0.len() < 2 {
                variant_ident = None;
            } else if let Some(ScopeUnit {
                kind: ScopeUnitKind::Variant(variant_id),
                ..
            }) = s.scope.0.get(s.scope.0.len() - 2)
            {
                variant_ident = Some(*variant_id);
            }

            if ss.len() > 1 && variant_ident.is_some() {
                let mut variant_ss = ss.clone();

                // TODO: check scope
                variant_ss.retain(|s| {
                    matches!(s.data, SymbolData::StructDef) && s.id == *ids.last().unwrap()
                    // && s.scope.0.starts_with(&variant_scope.0)
                });

                variant_ident = Some(variant_ident.unwrap());
            }

            s
        };

        let processed_node = match s.data {
            SymbolData::EnumComponent { enum_id, val } => {
                self.access_enum_component(s.id, enum_id, val, variant_ident, rhs)?
            }
            SymbolData::FunctionDef { .. } => self.access_func_def(&s, rhs)?,
            SymbolData::StructDef => self.access_struct_def(s.id, rhs)?,
            SymbolData::UnionDef => self.access_union_def(s.id, rhs)?,
            _ => todo!("Unaccessible: {:?}", s.data),
        };

        Ok(processed_node)
    }

    fn access_enum_component(
        &mut self,
        enum_component_id: Ident,
        enum_name: Ident,
        enum_val: usize,
        variant_kind: Option<Ident>,
        rhs: &Ast,
    ) -> Result<Option<Expression>, Message> {
        let location = rhs.location();

        if let Some(variant_name) = variant_kind {
            Ok(Some(Expression {
                location: rhs.location(),
                kind: ExpressionKind::Term {
                    node: Box::new(Ast::Value(Value {
                        location,
                        kind: ValueKind::Struct {
                            identifier: variant_name,
                            components: vec![
                                (
                                    variant_utils::get_variant_data_kind_field_id(),
                                    Ast::Value(Value {
                                        location,
                                        kind: ValueKind::Integer(enum_val),
                                    }),
                                ),
                                (
                                    variant_utils::get_variant_data_field_id(),
                                    Ast::Value(Value {
                                        location,
                                        kind: ValueKind::Struct {
                                            identifier:
                                                tanitc_ast::variant_utils::get_variant_data_type_id(
                                                    variant_name,
                                                ),
                                            components: vec![(
                                                enum_component_id,
                                                match rhs {
                                                    Ast::Value(Value {
                                                        kind: ValueKind::Identifier(_),
                                                        location,
                                                    }) => Ast::Value(Value {
                                                        location: *location,
                                                        kind: ValueKind::Struct {
                                                            identifier: Ident::from(format!(
                                                                "__{variant_name}__{enum_component_id}__"
                                                            )),
                                                            components: vec![],
                                                        },
                                                    }),
                                                    Ast::Value(Value {
                                                        kind: ValueKind::Struct { identifier, components },
                                                        location,
                                                    }) => Ast::Value(Value { location: *location, kind: ValueKind::Struct {
                                                        identifier: Ident::from(format!(
                                                            "__{variant_name}__{identifier}__"
                                                        )),
                                                        components: components.clone() } }),
                                                    Ast::Expression(Expression { location, ..}) => Ast::Value(Value {
                                                        location: *location,
                                                        kind: ValueKind::Integer(0),
                                                    }),
                                                    _ => return Err(Message::unreachable(
                                                        rhs.location(),
                                                        &format!("Unexpected value in access_enum_component ({rhs:?})"),
                                                    )),
                                                },
                                            )],
                                        },
                                    }),
                                ),
                            ],
                        },
                    })),
                    ty: Type::Custom(variant_name.to_string()),
                },
            }))
        } else {
            Ok(Some(Expression {
                location,
                kind: ExpressionKind::Term {
                    node: Box::new(Ast::Value(Value {
                        location,
                        kind: ValueKind::Integer(enum_val),
                    })),
                    ty: Type::Custom(enum_name.to_string()),
                },
            }))
        }
    }

    fn access_func_def(
        &mut self,
        func_symbol: &Symbol,
        rhs: &mut Ast,
    ) -> Result<Option<Expression>, Message> {
        if let Ast::Value(Value {
            location,
            kind:
                ValueKind::Call {
                    arguments: call_args,
                    ..
                },
        }) = rhs
        {
            self.analyze_call(func_symbol, call_args, *location)?;
        } else {
            todo!("Unexpected rhs: {rhs:#?}");
        };

        Ok(None)
    }

    fn get_struct_fields(&mut self, struct_name: Ident) -> Result<HashMap<Ident, Type>, Message> {
        let mut struct_comps = HashMap::<Ident, Type>::new();
        let mut ss = self.table.get_symbols();
        ss.retain(|s| matches!(s.data, SymbolData::StructField { .. }));
        for s in ss.iter() {
            let struct_field_id = s.id;
            if let SymbolData::StructField { struct_id, ty } = &s.data {
                if struct_name == *struct_id {
                    struct_comps.insert(struct_field_id, ty.clone());
                }
            }
        }
        Ok(struct_comps)
    }

    fn access_struct_def(
        &mut self,
        struct_name: Ident,
        node: &Ast,
    ) -> Result<Option<Expression>, Message> {
        let mut value = Box::new(node.clone());

        let struct_comps = self.get_struct_fields(struct_name)?;

        if let Ast::Value(Value {
            kind:
                ValueKind::Struct {
                    components: value_comps,
                    ..
                },
            ..
        }) = value.as_mut()
        {
            if let Err(mut msg) =
                self.check_struct_components(value_comps, struct_name, &struct_comps)
            {
                msg.location = node.location();
                return Err(msg);
            }
        } else {
            return Err(Message::unreachable(
                node.location(),
                "expected ValueKind::Struct, actually: {node:?}",
            ));
        }

        let node = Expression {
            location: node.location(),
            kind: ExpressionKind::Term {
                node: value,
                ty: Type::Custom(struct_name.to_string()),
            },
        };

        Ok(Some(node))
    }

    fn access_union_def(
        &mut self,
        union_name: Ident,
        node: &Ast,
    ) -> Result<Option<Expression>, Message> {
        let mut value = Box::new(node.clone());

        let union_comps = {
            let mut union_comps = HashMap::<Ident, Type>::new();
            let mut ss = self.table.get_symbols();
            ss.retain(|s| matches!(s.data, SymbolData::UnionField { .. }));
            for s in ss.iter() {
                let union_field_id = s.id;
                if let SymbolData::UnionField { union_id, ty } = &s.data {
                    if union_name == *union_id {
                        union_comps.insert(union_field_id, ty.clone());
                    }
                }
            }
            union_comps
        };

        if let Ast::Value(value) = value.as_mut() {
            let (union_id, value_comps) = if let ValueKind::Struct {
                identifier,
                components,
            } = &mut value.kind
            {
                (*identifier, std::mem::take(components))
            } else {
                return Err(Message::unreachable(
                    value.location,
                    "expected ValueKind::Struct",
                ));
            };

            if let Err(mut msg) =
                self.check_union_components(&value_comps, union_name, &union_comps)
            {
                msg.location = node.location();
                return Err(msg);
            }

            value.kind = ValueKind::Struct {
                identifier: union_id,
                components: value_comps,
            }
        } else {
            return Err(Message::unreachable(
                node.location(),
                "expected ValueKind::Struct, actually: {rhs:?}",
            ));
        }

        let node = Expression {
            location: node.location(),
            kind: ExpressionKind::Term {
                node: value,
                ty: Type::Custom(union_name.to_string()),
            },
        };

        Ok(Some(node))
    }

    fn preprocess_access_tree(
        ids: &mut Vec<Ident>,
        lhs: &mut Ast,
        rhs: &mut Ast,
    ) -> Result<(), Message> {
        let mut loc = Location::new();

        let lhs_id = match lhs {
            Ast::Value(Value {
                location,
                kind: ValueKind::Identifier(id),
            }) => {
                loc = *location;
                *id
            }
            _ => Ident::default(),
        };

        ids.push(lhs_id);

        match rhs {
            Ast::Expression(Expression {
                kind: ExpressionKind::Access { lhs, rhs },
                ..
            }) => {
                Self::preprocess_access_tree(ids, lhs, rhs)?;
            }
            Ast::Value(Value {
                kind: ValueKind::Identifier(rhs_id),
                ..
            }) => {
                ids.push(*rhs_id);
            }
            Ast::Value(Value {
                kind: ValueKind::Call { identifier, .. },
                ..
            }) => {
                ids.push(*identifier);
            }
            Ast::Value(Value {
                kind: ValueKind::Struct { identifier, .. },
                ..
            }) => {
                ids.push(*identifier);
            }
            _ => {
                return Err(Message::unreachable(
                    loc,
                    &format!(
                        "expected ExpressionKind::Access or Value::Identifier, actually: {rhs:?}"
                    ),
                ))
            }
        }

        Ok(())
    }

    fn analyze_conversion_expr(&mut self) -> Result<Option<Expression>, Message> {
        todo!()
    }

    fn analyze_term_expr(&mut self, node: &mut Box<Ast>) -> Result<Option<Expression>, Message> {
        node.accept_mut(self)?;
        Ok(None)
    }

    #[allow(dead_code)]
    fn convert_expr_node(&mut self, expr_node: &mut Ast) -> Result<(), Message> {
        if let Ast::Expression(node) = expr_node {
            let location = node.location;

            if let ExpressionKind::Binary {
                operation,
                lhs,
                rhs,
            } = &mut node.kind
            {
                self.convert_expr_node(lhs)?;
                self.convert_expr_node(rhs)?;

                let lhs_type = self.get_type(lhs);

                let rhs_type = self.get_type(rhs);

                let func_name_str = format!(
                    "__tanit_compiler__{}_{}_{}",
                    match operation {
                        BinaryOperation::Add => "add",
                        BinaryOperation::Sub => "sub",
                        BinaryOperation::Mul => "mul",
                        BinaryOperation::Div => "div",
                        BinaryOperation::Mod => "mod",
                        BinaryOperation::ShiftL => "lshift",
                        BinaryOperation::ShiftR => "rshift",
                        BinaryOperation::BitwiseOr => "or",
                        BinaryOperation::BitwiseAnd => "and",
                        _ => return Err(Message::new(location, "Unexpected operation")),
                    },
                    lhs_type,
                    rhs_type
                );

                let func_id = Ident::from(func_name_str);

                *expr_node = Ast::from(Value {
                    location,
                    kind: ValueKind::Call {
                        identifier: func_id,
                        arguments: vec![
                            CallArg {
                                location,
                                identifier: None,
                                kind: CallArgKind::Positional(0, lhs.clone()),
                            },
                            CallArg {
                                location,
                                identifier: None,
                                kind: CallArgKind::Positional(1, rhs.clone()),
                            },
                        ],
                    },
                });
            }

            Ok(())
        } else {
            unreachable!()
        }
    }
}

// Block
impl Analyzer {
    fn analyze_global_block(&mut self, block: &mut Block) -> Result<(), Message> {
        for n in block.statements.iter_mut() {
            let is_denied = matches!(
                n,
                Ast::ControlFlow(_)
                    | Ast::Block(_)
                    | Ast::Value(_)
                    | Ast::BranchStmt(_)
                    | Ast::Expression(_)
                    | Ast::TypeSpec(_)
            );

            if is_denied {
                self.scope.pop();

                return Err(Message {
                    location: n.location(),
                    text: format!("Node \"{}\" is not allowed in global scope", n.name()),
                });
            }

            if let Err(err) = n.accept_mut(self) {
                self.error(err);
            }
        }

        Ok(())
    }

    fn analyze_local_block(&mut self, block: &mut Block) -> Result<(), Message> {
        let cnt = self.counter();

        let safety = self.get_current_safety();

        self.scope.push(ScopeUnit {
            kind: ScopeUnitKind::Block(cnt),
            safety: block.attrs.safety.unwrap_or(safety),
        });

        for n in block.statements.iter_mut() {
            let is_denied = matches!(
                n,
                Ast::StructDef(_)
                    | Ast::UnionDef(_)
                    | Ast::VariantDef(_)
                    | Ast::FuncDef(_)
                    | Ast::AliasDef(_)
                    | Ast::EnumDef(_)
            );

            if is_denied {
                self.scope.pop();

                return Err(Message {
                    location: n.location(),
                    text: format!("Node \"{}\" is not allowed in local scope", n.name()),
                });
            }

            if let Err(err) = n.accept_mut(self) {
                self.error(err);
            }
        }

        self.scope.pop();

        Ok(())
    }
}

// Struct value
impl Analyzer {
    fn analyze_struct_value(&mut self, value: &mut Value) -> Result<(), Message> {
        let ValueKind::Struct {
            identifier: object_name,
            components: value_comps,
        } = &mut value.kind
        else {
            return Err(Message::unreachable(
                value.location,
                "Expected ValueKind::Struct",
            ));
        };

        let mut object = if let Some(symbol) = self.get_first_symbol(*object_name) {
            symbol
        } else {
            return Err(Message::undefined_id(value.location, *object_name));
        };

        if let SymbolData::AliasDef { ty } = &object.data {
            match ty {
                Type::Custom(id) => {
                    let alias_to_id = Ident::from(id.clone());

                    if let Some(symbol) = self.get_first_symbol(alias_to_id) {
                        object = symbol;
                    } else {
                        return Err(Message::undefined_id(value.location, alias_to_id));
                    };
                }
                ty if ty.is_common() => {
                    return Err(Message {
                        location: value.location,
                        text: format!("Common type \"{ty}\" does not have any fields"),
                    })
                }
                _ => {
                    todo!("Unexpected type: {ty}");
                }
            }
        }

        if matches!(object.data, SymbolData::StructDef) {
            let mut struct_comps = HashMap::<Ident, Type>::new();
            let mut ss = self.table.get_symbols();

            ss.retain(|s| matches!(s.data, SymbolData::StructField { .. }));
            for s in ss.iter() {
                if let SymbolData::StructField { struct_id, ty } = &s.data {
                    if *struct_id == object.id {
                        struct_comps.insert(s.id, ty.clone());
                    }
                }
            }

            if let Err(mut msg) =
                self.check_struct_components(value_comps, *object_name, &struct_comps)
            {
                msg.location = value.location;
                return Err(msg);
            }
        } else if matches!(object.data, SymbolData::UnionDef) {
            let mut union_comps = HashMap::<Ident, Type>::new();
            let mut ss = self.table.get_symbols();

            ss.retain(|s| matches!(s.data, SymbolData::UnionField { .. }));
            for s in ss.iter() {
                if let SymbolData::UnionField { union_id, ty } = &s.data {
                    if *union_id == object.id {
                        union_comps.insert(s.id, ty.clone());
                    }
                }
            }

            if let Err(mut msg) =
                self.check_union_components(value_comps, *object_name, &union_comps)
            {
                msg.location = value.location;
                return Err(msg);
            }

            value.kind = ValueKind::Struct {
                identifier: *object_name,
                components: std::mem::take(value_comps),
            };
        } else {
            return Err(Message::new(
                value.location,
                &format!("Cannot find struct or union named \"{object_name}\" in this scope"),
            ));
        }

        Ok(())
    }

    fn check_struct_components(
        &mut self,
        value_comps: &[(Ident, Ast)],
        struct_name: Ident,
        struct_comps: &HashMap<Ident, Type>,
    ) -> Result<(), Message> {
        if value_comps.len() != struct_comps.len() {
            return Err(Message::new(
                Location::new(),
                &format!(
                    "Struct \"{}\" consists of {} fields, but {} were supplied",
                    struct_name,
                    struct_comps.len(),
                    value_comps.len()
                ),
            ));
        }

        for comp_id in 0..value_comps.len() {
            let value_comp = value_comps.get(comp_id).unwrap();
            let value_comp_name = value_comp.0;
            let value_comp_type = self.get_type(&value_comp.1);
            let struct_comp_type = struct_comps.get(&value_comp_name).unwrap();

            let mut alias_to = self.find_alias_value(struct_comp_type);

            if value_comp_type == *struct_comp_type {
                alias_to = None;
            }

            if alias_to.is_none() && value_comp_type != *struct_comp_type {
                return Err(Message {
                    location: value_comp.1.location(),
                    text: format!(
                        "Struct field named \"{value_comp_name}\" is {struct_comp_type}, but initialized like {value_comp_type}",
                    ),
                });
            } else if alias_to.as_ref().is_some_and(|ty| value_comp_type != *ty) {
                return Err(Message {
                    location: value_comp.1.location(),
                    text: format!(
                        "Struct field named \"{value_comp_name}\" is {struct_comp_type} (aka: {}), but initialized like {value_comp_type}",
                        alias_to.unwrap()
                    ),
                });
            }
        }

        Ok(())
    }

    fn check_union_components(
        &mut self,
        value_comps: &[(Ident, Ast)],
        union_name: Ident,
        union_comps: &HashMap<Ident, Type>,
    ) -> Result<(), Message> {
        let union_comp_size = union_comps.len();
        let initialized_comp_size = value_comps.len();

        if union_comp_size == 0 && initialized_comp_size > 0 {
            return Err(Message::new(
                Location::new(),
                &format!(
                    "Union \"{union_name}\" has no fields, but were supplied {initialized_comp_size} fields",
                ),
            ));
        }

        if union_comp_size > 0 && initialized_comp_size > 1 {
            return Err(Message::new(
                Location::new(),
                &format!(
                    "Only one union field must be initialized, but {initialized_comp_size} were initialized",
                ),
            ));
        }

        for comp_id in 0..initialized_comp_size {
            let value_comp = value_comps.get(comp_id).unwrap();
            let value_comp_name = value_comp.0;
            let value_comp_type = self.get_type(&value_comp.1);
            let union_comp_type = union_comps.get(&value_comp.0).unwrap();

            let mut alias_to = self.find_alias_value(union_comp_type);

            if value_comp_type == *union_comp_type {
                alias_to = None;
            }

            if alias_to.is_none() && value_comp_type != *union_comp_type {
                return Err(Message::new(
                    Location::new(),
                    &format!(
                        "Union field named \"{value_comp_name}\" is {union_comp_type}, but initialized like {value_comp_type}"
                    ),
                ));
            } else if alias_to.as_ref().is_some_and(|ty| value_comp_type != *ty) {
                return Err(Message::new(
                        Location::new(),
                        &format!(
                            "Union field named \"{value_comp_name}\" is {union_comp_type} (aka: {}), but initialized like {value_comp_type}",
                            alias_to.unwrap()
                        ),
                    ));
            }
        }

        Ok(())
    }
}

// Variant
impl Analyzer {
    fn analyze_variant_def(&mut self, variant_def: &mut VariantDef) -> Result<(), Message> {
        if self.has_symbol(variant_def.identifier) {
            return Err(Message::multiple_ids(
                variant_def.location,
                variant_def.identifier,
            ));
        }

        let safety = self.get_current_safety();

        self.scope.push(ScopeUnit {
            kind: ScopeUnitKind::Variant(variant_def.identifier),
            safety,
        });

        for internal in variant_def.internals.iter_mut() {
            internal.accept_mut(self)?;
        }

        self.create_variant_data_kind_symbol(variant_def.identifier, &variant_def.fields, safety)?;

        self.create_variant_data_symbols(variant_def.identifier, &variant_def.fields, safety)?;

        self.scope.pop();
        self.add_symbol(self.create_symbol(variant_def.identifier, SymbolData::StructDef));

        Ok(())
    }

    fn create_variant_data_kind_symbol(
        &mut self,
        variant_id: Ident,
        fields: &BTreeMap<Ident, VariantField>,
        safety: Safety,
    ) -> Result<(), Message> {
        let data_kind_id = tanitc_ast::variant_utils::get_variant_data_kind_id(variant_id);

        self.scope.push(ScopeUnit {
            kind: ScopeUnitKind::Enum(data_kind_id),
            safety,
        });

        for (field_num, (field_id, _)) in fields.iter().enumerate() {
            self.add_symbol(self.create_symbol(
                *field_id,
                SymbolData::EnumComponent {
                    enum_id: data_kind_id,
                    val: field_num,
                },
            ));
        }

        self.scope.pop();

        Ok(())
    }

    fn create_variant_common_field_symbol(&mut self, field_id: Ident) -> Result<(), Message> {
        self.add_symbol(self.create_symbol(field_id, SymbolData::StructDef));
        Ok(())
    }

    fn create_variant_struct_field_symbol(
        &mut self,
        field_id: Ident,
        subfields: &BTreeMap<Ident, TypeSpec>,
        safety: Safety,
    ) -> Result<(), Message> {
        self.scope.push(ScopeUnit {
            kind: ScopeUnitKind::Struct(field_id),
            safety,
        });

        for (subfield_id, subfield_type) in subfields.iter() {
            self.add_symbol(self.create_symbol(
                *subfield_id,
                SymbolData::StructField {
                    struct_id: field_id,
                    ty: subfield_type.get_type(),
                },
            ));
        }

        self.scope.pop();
        self.add_symbol(self.create_symbol(field_id, SymbolData::StructDef));

        Ok(())
    }

    fn create_variant_tuple_field_symbol(
        &mut self,
        field_id: Ident,
        components: &[TypeSpec],
        safety: Safety,
    ) -> Result<(), Message> {
        self.scope.push(ScopeUnit {
            kind: ScopeUnitKind::Struct(field_id),
            safety,
        });

        for (field_num, field_type) in components.iter().enumerate() {
            let subfield_id = Ident::from(format!("_{field_num}"));
            self.add_symbol(self.create_symbol(
                subfield_id,
                SymbolData::StructField {
                    struct_id: field_id,
                    ty: field_type.get_type(),
                },
            ));
        }

        self.scope.pop();
        self.add_symbol(self.create_symbol(field_id, SymbolData::StructDef));

        Ok(())
    }

    fn create_variant_fields_symbols(
        &mut self,
        fields: &BTreeMap<Ident, VariantField>,
        safety: Safety,
    ) -> Result<(), Message> {
        for (field_id, field_data) in fields.iter() {
            match field_data {
                VariantField::Common => self.create_variant_common_field_symbol(*field_id)?,
                VariantField::StructLike(subfields) => {
                    self.create_variant_struct_field_symbol(*field_id, subfields, safety)?
                }
                VariantField::TupleLike(components) => {
                    self.create_variant_tuple_field_symbol(*field_id, components, safety)?
                }
            }
        }
        Ok(())
    }

    fn create_variant_data_symbols(
        &mut self,
        variant_id: Ident,
        fields: &BTreeMap<Ident, VariantField>,
        safety: Safety,
    ) -> Result<(), Message> {
        let union_id = tanitc_ast::variant_utils::get_variant_data_type_id(variant_id);

        self.scope.push(ScopeUnit {
            kind: ScopeUnitKind::Union(union_id),
            safety,
        });
        self.create_variant_fields_symbols(fields, safety)?;
        self.scope.pop();

        Ok(())
    }
}

fn get_ordinal_number_suffix(num: usize) -> &'static str {
    match num % 10 {
        0 => "st",
        1 => "nd",
        2 => "rd",
        _ => "th",
    }
}
