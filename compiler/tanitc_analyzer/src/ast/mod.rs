use tanitc_ast::{
    self,
    expression_utils::{BinaryOperation, UnaryOperation},
    AliasDef, Ast, Block, Branch, BranchKind, CallArg, CallArgKind, ControlFlow, ControlFlowKind,
    EnumDef, Expression, ExpressionKind, ExternDef, FunctionDef, ModuleDef, StructDef, TypeSpec,
    UnionDef, Use, Value, ValueKind, VariableDef, VariantDef, VisitorMut,
};
use tanitc_ident::Ident;
use tanitc_lexer::location::Location;
use tanitc_messages::Message;
use tanitc_symbol_table::{
    entry::{
        AliasDefData, Entry, EnumData, EnumDefData, FuncDefData, ModuleDefData, StructDefData,
        StructFieldData, SymbolKind, UnionDefData, VarDefData, VarStorageType,
    },
    table::Table,
};
use tanitc_ty::Type;

use std::{cmp::Ordering, collections::BTreeMap};

use crate::Analyzer;

pub mod variants;

impl VisitorMut for Analyzer {
    fn visit_module_def(&mut self, module_def: &mut ModuleDef) -> Result<(), Message> {
        if self.has_symbol(module_def.identifier) {
            return Err(Message::multiple_ids(
                module_def.location,
                module_def.identifier,
            ));
        }

        self.table.insert(Entry {
            name: module_def.identifier,
            is_static: true,
            kind: SymbolKind::from(ModuleDefData {
                table: Box::new(Table::new()),
            }),
        });

        let mut analyzer = Analyzer::with_options(self.compile_options.clone());

        if let Some(body) = &mut module_def.body {
            analyzer.visit_block(body)?;
            let entry = self.table.lookup_mut(module_def.identifier).unwrap();
            let SymbolKind::ModuleDef(ref mut data) = &mut entry.kind else {
                unreachable!();
            };

            data.table = analyzer.table;
        }

        Ok(())
    }

    fn visit_struct_def(&mut self, struct_def: &mut StructDef) -> Result<(), Message> {
        if self.has_symbol(struct_def.identifier) {
            return Err(Message::multiple_ids(
                struct_def.location,
                struct_def.identifier,
            ));
        }

        for internal in struct_def.internals.iter_mut() {
            internal.accept_mut(self)?;
        }

        let mut fields = BTreeMap::<Ident, StructFieldData>::new();
        for (field_id, field_ty) in struct_def.fields.iter() {
            fields.insert(
                *field_id,
                StructFieldData {
                    ty: field_ty.get_type(),
                },
            );
        }

        self.add_symbol(Entry {
            name: struct_def.identifier,
            is_static: true,
            kind: SymbolKind::from(StructDefData { fields }),
        });

        Ok(())
    }

    fn visit_union_def(&mut self, union_def: &mut UnionDef) -> Result<(), Message> {
        if self.has_symbol(union_def.identifier) {
            return Err(Message::multiple_ids(
                union_def.location,
                union_def.identifier,
            ));
        }

        for internal in union_def.internals.iter_mut() {
            internal.accept_mut(self)?;
        }

        let mut fields = BTreeMap::<Ident, StructFieldData>::new();
        for (field_id, field_ty) in union_def.fields.iter() {
            fields.insert(
                *field_id,
                StructFieldData {
                    ty: field_ty.get_type(),
                },
            );
        }

        self.add_symbol(Entry {
            name: union_def.identifier,
            is_static: true,
            kind: SymbolKind::from(UnionDefData { fields }),
        });

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
        let mut enums = BTreeMap::<Ident, Entry>::new();
        for field in enum_def.fields.iter_mut() {
            if let Some(value) = field.1 {
                counter = *value;
            }

            // mark unmarked enum fields
            *field.1 = Some(counter);

            enums.insert(
                *field.0,
                Entry {
                    name: *field.0,
                    is_static: true,
                    kind: SymbolKind::Enum(EnumData {
                        enum_name: enum_def.identifier,
                        value: counter,
                    }),
                },
            );

            counter += 1;
        }

        self.add_symbol(Entry {
            name: enum_def.identifier,
            is_static: true,
            kind: SymbolKind::from(EnumDefData { enums }),
        });

        Ok(())
    }

    fn visit_func_def(&mut self, func_def: &mut FunctionDef) -> Result<(), Message> {
        if self.has_symbol(func_def.identifier) {
            return Err(Message::multiple_ids(
                func_def.location,
                func_def.identifier,
            ));
        }

        let mut scope_info = self.table.get_scope_info();
        scope_info.safety = func_def.attrs.safety.unwrap_or(self.get_current_safety());
        scope_info.is_in_func = true;

        self.table.enter_scope(scope_info);

        let mut parameters = Vec::<(Ident, Type)>::new();
        for p in func_def.parameters.iter_mut() {
            let Ast::VariableDef(param_def) = p else {
                return Err(Message::from_string(
                    p.location(),
                    format!("Unexpected param node type: {}", p.name()),
                ));
            };

            if let Err(err) = self.visit_variable_def(param_def) {
                self.error(err);
            } else {
                parameters.push((param_def.identifier, param_def.var_type.get_type()));
            }
        }

        if let Some(body) = &mut func_def.body {
            body.accept_mut(self)?;
        }

        self.table.exit_scope();

        self.add_symbol(Entry {
            name: func_def.identifier,
            is_static: false,
            kind: SymbolKind::from(FuncDefData {
                parameters,
                return_type: func_def.return_type.get_type(),
                is_virtual: false,
                is_inline: false,
                no_return: func_def.return_type.get_type() == Type::unit(),
            }),
        });

        Ok(())
    }

    fn visit_extern_def(&mut self, extern_def: &mut ExternDef) -> Result<(), Message> {
        for func_def in extern_def.functions.iter_mut() {
            if let Err(err) = self.visit_func_def(func_def) {
                self.error(err);
            }
        }

        Ok(())
    }

    fn visit_variable_def(&mut self, var_def: &mut VariableDef) -> Result<(), Message> {
        if self.has_symbol(var_def.identifier) {
            return Err(Message::multiple_ids(var_def.location, var_def.identifier));
        }

        self.add_symbol(Entry {
            name: var_def.identifier,
            is_static: false,
            kind: SymbolKind::from(VarDefData {
                var_type: var_def.var_type.get_type(),
                is_mutable: var_def.is_mutable,
                is_initialization: false,
                storage: VarStorageType::Auto,
            }),
        });

        Ok(())
    }

    fn visit_alias_def(&mut self, alias_def: &mut AliasDef) -> Result<(), Message> {
        if self.has_symbol(alias_def.identifier) {
            return Err(Message::multiple_ids(
                alias_def.location,
                alias_def.identifier,
            ));
        }

        self.add_symbol(Entry {
            name: alias_def.identifier,
            is_static: true,
            kind: SymbolKind::AliasDef(AliasDefData {
                ty: alias_def.value.get_type(),
            }),
        });

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

        let mut scope_info = self.table.get_scope_info();

        match &mut branch.kind {
            BranchKind::While { body, condition } => {
                scope_info.is_in_loop = true;
                self.table.enter_scope(scope_info);

                condition.accept_mut(self)?;

                analyze_condition(condition.as_mut(), self)?;
                analyze_body(body.as_mut(), self)?;

                self.table.exit_scope();

                Ok(())
            }
            BranchKind::Loop { body } => {
                scope_info.is_in_loop = true;
                self.table.enter_scope(scope_info);

                analyze_body(body.as_mut(), self)?;

                self.table.exit_scope();

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
        let is_in_func = self.table.get_scope_info().is_in_func;
        let is_in_loop = self.table.get_scope_info().is_in_loop;

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
                let Some(func_entry) = self.table.lookup(*func_name) else {
                    return Err(Message::undefined_id(val.location, *func_name));
                };

                let SymbolKind::FuncDef(func_data) = func_entry.kind.clone() else {
                    return Err(Message::undefined_func(val.location, *func_name));
                };

                self.analyze_call(func_entry.name, &func_data, call_args, val.location)?;

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
                if let Some(entry) = self.table.lookup(*id) {
                    if let SymbolKind::VarDef(data) = &entry.kind {
                        return data.var_type.clone();
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
                if let Some(ss) = self.table.lookup(*identifier) {
                    if let SymbolKind::FuncDef(data) = &ss.kind {
                        return data.return_type.clone();
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
        func_name: Ident,
        func_data: &FuncDefData,
        call_args: &mut [CallArg],
        location: Location,
    ) -> Result<(), Message> {
        if func_name.is_built_in() {
            return Ok(());
        }

        let params = &func_data.parameters;

        self.check_arg_count(func_name, call_args, params, location)?;

        let mut positional_skipped = false;
        for call_arg in call_args.iter_mut() {
            if let Err(err) = self.analyze_arg(func_name, params, call_arg, &mut positional_skipped)
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

            let entry = self.table.lookup(type_id)?;

            let SymbolKind::AliasDef(alias_data) = &entry.kind else {
                return None;
            };

            if let Some(alias_to) = self.find_alias_value(&alias_data.ty) {
                Some(alias_to)
            } else {
                Some(alias_data.ty.clone())
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
            let Some(entry) = self.table.lookup(*id) else {
                return Err(Message::undefined_id(*location, *id));
            };

            if let SymbolKind::VarDef(var_data) = &entry.kind {
                if !var_data.is_mutable && does_mutate {
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

            self.add_symbol(Entry {
                name: node.identifier,
                is_static: false,
                kind: SymbolKind::from(VarDefData {
                    storage: VarStorageType::Auto,
                    var_type: node.var_type.get_type(),
                    is_mutable: node.is_mutable,
                    is_initialization: true,
                }),
            });
        } else if let Ast::Value(node) = lhs {
            match &node.kind {
                ValueKind::Identifier(id) => {
                    if let Some(entry) = self.table.lookup(*id) {
                        if let SymbolKind::VarDef(var_data) = &entry.kind {
                            if let Type::Ref { is_mutable, .. } = &var_data.var_type {
                                if !*is_mutable && does_mutate {
                                    self.error(Message::new(
                                        Location::new(),
                                        &format!(
                                            "Reference \"{id}\" is immutable in current scope",
                                        ),
                                    ));
                                }
                            } else if !var_data.is_mutable && does_mutate {
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

        let rhs = Self::preprocess_access_tree(&mut ids, lhs, rhs)?;

        let Some(entry) = self.table.lookup_qualified(ids.iter().peekable()) else {
            return Err(Message::undefined_id(rhs.location(), *ids.last().unwrap()));
        };

        let entry = entry.clone();

        let processed_node = match &entry.kind {
            SymbolKind::Enum(data) => self.access_enum(data, rhs)?,
            SymbolKind::Variant(data) => self.access_variant(entry.name, data, rhs)?,
            SymbolKind::StructDef(data) => self.access_struct_def(entry.name, data, rhs)?,
            SymbolKind::UnionDef(data) => self.access_union_def(entry.name, data, rhs)?,
            SymbolKind::FuncDef(data) => self.access_func_def(entry.name, data, rhs)?,
            _ => todo!("Unaccessible: {:?}", entry.kind),
        };

        Ok(processed_node)
    }

    fn access_enum(
        &mut self,
        enum_data: &EnumData,
        rhs: &Ast,
    ) -> Result<Option<Expression>, Message> {
        let location = rhs.location();

        Ok(Some(Expression {
            location,
            kind: ExpressionKind::Term {
                node: Box::new(Ast::Value(Value {
                    location,
                    kind: ValueKind::Integer(enum_data.value),
                })),
                ty: Type::Custom(enum_data.enum_name.to_string()),
            },
        }))
    }

    fn access_func_def(
        &mut self,
        func_name: Ident,
        func_data: &FuncDefData,
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
            self.analyze_call(func_name, func_data, call_args, *location)?;
        } else {
            todo!("Unexpected rhs: {rhs:#?}");
        };

        Ok(None)
    }

    fn access_struct_def(
        &mut self,
        struct_name: Ident,
        struct_data: &StructDefData,
        node: &Ast,
    ) -> Result<Option<Expression>, Message> {
        let mut value = Box::new(node.clone());

        let struct_comps = &struct_data.fields;

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
                self.check_struct_components(value_comps, struct_name, struct_comps)
            {
                msg.location = node.location();
                return Err(msg);
            }
        } else {
            return Err(Message::unreachable(
                node.location(),
                format!("expected ValueKind::Struct, actually: {node:?}"),
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
        union_data: &UnionDefData,
        node: &Ast,
    ) -> Result<Option<Expression>, Message> {
        let mut value_clone = Box::new(node.clone());

        let union_comps = &union_data.fields;

        let Ast::Value(value) = value_clone.as_mut() else {
            return Err(Message::unreachable(
                node.location(),
                format!("expected Ast::Value, actually: {}", node.name()),
            ));
        };

        let ValueKind::Struct {
            identifier: union_id,
            components: value_comps,
        } = &mut value.kind
        else {
            return Err(Message::unreachable(
                value.location,
                format!("expected ValueKind::Struct, actually: {:?}", value.kind),
            ));
        };

        if let Err(mut msg) = self.check_union_components(value_comps, union_name, union_comps) {
            msg.location = node.location();
            return Err(msg);
        }

        value.kind = ValueKind::Struct {
            identifier: *union_id,
            components: std::mem::take(value_comps),
        };

        let node = Expression {
            location: node.location(),
            kind: ExpressionKind::Term {
                node: value_clone,
                ty: Type::Custom(union_name.to_string()),
            },
        };

        Ok(Some(node))
    }

    fn preprocess_access_tree<'a>(
        ids: &mut Vec<Ident>,
        lhs: &'a Ast,
        rhs: &'a mut Ast,
    ) -> Result<&'a mut Ast, Message> {
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
            }) => Self::preprocess_access_tree(ids, lhs, rhs),
            Ast::Value(Value {
                kind: ValueKind::Identifier(rhs_id),
                ..
            }) => {
                ids.push(*rhs_id);
                Ok(rhs)
            }
            Ast::Value(Value {
                kind: ValueKind::Call { identifier, .. },
                ..
            }) => {
                ids.push(*identifier);
                Ok(rhs)
            }
            Ast::Value(Value {
                kind: ValueKind::Struct { identifier, .. },
                ..
            }) => {
                ids.push(*identifier);
                Ok(rhs)
            }
            _ => Err(Message::unreachable(
                loc,
                format!("Expected ExpressionKind::Access or Value::Identifier, actually: {rhs:?}"),
            )),
        }
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
                self.error(Message {
                    location: n.location(),
                    text: format!("Node \"{}\" is not allowed in global scope", n.name()),
                });

                continue;
            }

            if let Err(err) = n.accept_mut(self) {
                self.error(err);
            }
        }

        Ok(())
    }

    fn analyze_local_block(&mut self, block: &mut Block) -> Result<(), Message> {
        let mut scope_info = self.table.get_scope_info();
        if let Some(safety) = &block.attrs.safety {
            scope_info.safety = *safety;
        }

        self.table.enter_scope(scope_info);

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
                self.error(Message {
                    location: n.location(),
                    text: format!("Node \"{}\" is not allowed in local scope", n.name()),
                });

                continue;
            }

            if let Err(err) = n.accept_mut(self) {
                self.error(err);
            }
        }

        self.table.exit_scope();

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
                format!("Expected ValueKind::Struct, actually: {:?}", value.kind),
            ));
        };

        let mut object = if let Some(entry) = self.table.lookup(*object_name) {
            entry.clone()
        } else {
            return Err(Message::undefined_id(value.location, *object_name));
        };

        if let SymbolKind::AliasDef(alias_data) = &object.kind {
            let ty = &alias_data.ty;
            match ty {
                Type::Custom(id) => {
                    let alias_to_id = Ident::from(id.clone());

                    if let Some(entry) = self.table.lookup(alias_to_id) {
                        object = entry.clone();
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

        match &object.kind {
            SymbolKind::StructDef(struct_data) => {
                if let Err(mut msg) =
                    self.check_struct_components(value_comps, *object_name, &struct_data.fields)
                {
                    msg.location = value.location;
                    return Err(msg);
                }
            }
            SymbolKind::UnionDef(union_data) => {
                if let Err(mut msg) =
                    self.check_union_components(value_comps, *object_name, &union_data.fields)
                {
                    msg.location = value.location;
                    return Err(msg);
                }

                value.kind = ValueKind::Struct {
                    identifier: *object_name,
                    components: std::mem::take(value_comps),
                };
            }
            _ => {
                return Err(Message::new(
                    value.location,
                    &format!("Cannot find struct or union named \"{object_name}\" in this scope"),
                ));
            }
        }

        Ok(())
    }

    fn check_struct_components(
        &mut self,
        value_comps: &[(Ident, Ast)],
        struct_name: Ident,
        struct_comps: &BTreeMap<Ident, StructFieldData>,
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
            let struct_comp_type = &struct_comps.get(&value_comp_name).unwrap().ty;

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
        union_comps: &BTreeMap<Ident, StructFieldData>,
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
            let union_comp_data = union_comps.get(&value_comp.0).unwrap();
            let union_comp_type = &union_comp_data.ty;

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

    fn check_tuple_components(
        &mut self,
        value_comps: &[Ast],
        tuple_name: Option<Ident>,
        tuple_comps: &BTreeMap<usize, StructFieldData>,
    ) -> Result<(), Message> {
        if value_comps.len() != tuple_comps.len() {
            return Err(Message::new(
                Location::new(),
                &format!(
                    "Tuple {} consists of {} fields, but {} were supplied",
                    if let Some(tuple_name) = tuple_name {
                        tuple_name.to_string()
                    } else {
                        "".to_string()
                    },
                    tuple_comps.len(),
                    value_comps.len()
                ),
            ));
        }

        for comp_id in 0..value_comps.len() {
            let value_comp = value_comps.get(comp_id).unwrap();
            let value_comp_type = self.get_type(value_comp);
            let tuple_comp_type = &tuple_comps.get(&comp_id).unwrap().ty;

            let mut alias_to = self.find_alias_value(tuple_comp_type);

            if value_comp_type == *tuple_comp_type {
                alias_to = None;
            }

            if alias_to.is_none() && value_comp_type != *tuple_comp_type {
                return Err(Message {
                    location: value_comp.location(),
                    text: format!(
                        "Tuple component with index \"{comp_id}\" is {tuple_comp_type}, but initialized like {value_comp_type}",
                    ),
                });
            } else if alias_to.as_ref().is_some_and(|ty| value_comp_type != *ty) {
                return Err(Message {
                    location: value_comp.location(),
                    text: format!(
                        "Tuple component with index \"{comp_id}\" is {tuple_comp_type} (aka: {}), but initialized like {value_comp_type}",
                        alias_to.unwrap()
                    ),
                });
            }
        }

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
