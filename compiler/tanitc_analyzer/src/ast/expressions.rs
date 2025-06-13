use tanitc_ast::{
    expression_utils::{BinaryOperation, UnaryOperation},
    Ast, CallArg, CallArgKind, Expression, ExpressionKind, Value, ValueKind,
};
use tanitc_ident::Ident;
use tanitc_lexer::location::Location;
use tanitc_messages::Message;
use tanitc_symbol_table::entry::{
    EnumData, FuncDefData, StructDefData, SymbolKind, UnionDefData, VarDefData,
};
use tanitc_ty::Type;

use crate::Analyzer;

impl Analyzer {
    pub fn analyze_unary_expr(
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

    pub fn analyze_binary_expr(
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
            self.analyze_variable_def(node, Some(rhs))?;
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
                                self.error(Message::const_mutation(node.location, *id));
                            }
                        }
                    }
                }
                ValueKind::Integer(..) | ValueKind::Decimal(..) => {}
                ValueKind::Text(..) => self.error(Message::new(
                    node.location,
                    "Cannot perform operation with text in this context",
                )),
                ValueKind::Array { .. } => self.error(Message::new(
                    node.location,
                    "Cannot perform operation with array in this context",
                )),
                ValueKind::Tuple { .. } => self.error(Message::new(
                    node.location,
                    "Cannot perform operation with tuple in this context",
                )),
                _ => self.error(Message::new(
                    node.location,
                    "Cannot perform operation with this object",
                )),
            }
        } else {
            lhs.accept_mut(self)?;
            let lhs_type = self.get_type(lhs);

            if lhs_type != rhs_type {
                self.error(Message::from_string(
                    rhs.location(),
                    format!(
                        "Cannot perform operation on objects with different types: {:?} and {:?}",
                        lhs_type, rhs_type
                    ),
                ));
            }
        }

        Ok(None)
    }

    pub fn analyze_access_expr(
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

    pub fn analyze_member_access(&mut self, lhs: &mut Ast, rhs: &mut Ast) -> Result<(), Message> {
        let mut ids = Vec::<Ident>::new();

        let rhs = Self::preprocess_access_tree(&mut ids, lhs, rhs)?;

        let Some(entry) = self.table.lookup_qualified(ids.iter().peekable()) else {
            return Err(Message::undefined_id(rhs.location(), *ids.last().unwrap()));
        };

        let entry = entry.clone();

        match &entry.kind {
            SymbolKind::VarDef(data) => self.check_member_access(entry.name, data, rhs),
            _ => unimplemented!("Entry.kind: {:?}", entry.kind),
        }
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

    fn check_member_access(
        &mut self,
        var_name: Ident,
        var_data: &VarDefData,
        node: &mut Ast,
    ) -> Result<(), Message> {
        let location = node.location();
        let var_type = &var_data.var_type;

        match node {
            Ast::Expression(Expression {
                kind:
                    ExpressionKind::Binary {
                        operation,
                        lhs,
                        rhs,
                    },
                ..
            }) => {
                if operation.does_mutate() && !var_data.is_mutable {
                    return Err(Message::const_mutation(location, var_name));
                }

                let Ast::Value(Value {
                    kind: ValueKind::Identifier(member_name),
                    ..
                }) = lhs.as_ref()
                else {
                    return Err(Message::unreachable(
                        location,
                        format!("Expected member access, actually: {lhs:?}"),
                    ));
                };

                let Type::Custom(struct_name) = var_type else {
                    return Err(Message::from_string(
                        location,
                        format!("Type \"{var_type}\" doesn't have any members, including \"{member_name}\""),
                    ));
                };

                let struct_name = Ident::from(struct_name.clone());
                let Some(struct_entry) = self.table.lookup(struct_name) else {
                    return Err(Message::new(
                        location,
                        "Struct type should be known at this point",
                    ));
                };

                let struct_entry = struct_entry.clone();

                match &struct_entry.kind {
                    SymbolKind::StructDef(struct_data) => {
                        let Some(field) = struct_data.fields.get(member_name) else {
                            return Err(Message::from_string(
                                location,
                                format!(
                                    "Struct \"{struct_name}\" doesn't have field \"{member_name}\""
                                ),
                            ));
                        };

                        rhs.accept_mut(self)?;

                        let rhs_type = self.get_type(rhs);
                        let field_type = &field.ty;

                        if *field_type != rhs_type {
                            return Err(Message::from_string(
                                rhs.location(),
                                format!(
                                    "Cannot perform operation on objects with different types: {field_type:?} and {rhs_type:?}",
                                ),
                            ));
                        }
                    }
                    SymbolKind::UnionDef(union_data) => {
                        let Some(field) = union_data.fields.get(member_name) else {
                            return Err(Message::from_string(
                                location,
                                format!(
                                    "Union \"{struct_name}\" doesn't have field \"{member_name}\""
                                ),
                            ));
                        };

                        rhs.accept_mut(self)?;

                        let rhs_type = self.get_type(rhs);
                        let field_type = &field.ty;

                        if *field_type != rhs_type {
                            return Err(Message::from_string(
                                rhs.location(),
                                format!(
                                    "Cannot perform operation on objects with different types: {field_type:?} and {rhs_type:?}",
                                ),
                            ));
                        }
                    }
                    _ => {}
                }
            }
            _ => todo!("{node:?}"),
        }

        Ok(())
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
            Ast::Expression(Expression {
                kind:
                    ExpressionKind::Binary {
                        operation: BinaryOperation::Assign,
                        ..
                    },
                ..
            }) => Ok(rhs),
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

    pub fn analyze_conversion_expr(&mut self) -> Result<Option<Expression>, Message> {
        todo!()
    }

    pub fn analyze_term_expr(
        &mut self,
        node: &mut Box<Ast>,
    ) -> Result<Option<Expression>, Message> {
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
