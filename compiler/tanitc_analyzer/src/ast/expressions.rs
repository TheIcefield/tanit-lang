use std::{iter::Peekable, slice::Iter};

use tanitc_ast::{
    expression_utils::{BinaryOperation, UnaryOperation},
    Ast, CallArg, CallArgKind, Expression, ExpressionKind, Value, ValueKind,
};
use tanitc_ident::Ident;
use tanitc_messages::{location::Location, Message};
use tanitc_symbol_table::{
    entry::{EnumData, FuncDefData, StructDefData, SymbolKind, UnionDefData, VarDefData},
    type_info::TypeInfo,
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
                if var_data.mutability.is_const() && does_mutate {
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

        let does_mutate = operation.does_mutate();

        if let Ast::VariableDef(node) = lhs {
            self.analyze_variable_def(node, Some(rhs))?;
        } else if let Ast::Value(node) = lhs {
            match &node.kind {
                ValueKind::Identifier(id) => {
                    let Some(entry) = self.table.lookup(*id) else {
                        return Err(Message::undefined_id(node.location, *id));
                    };

                    let SymbolKind::VarDef(var_data) = &entry.kind else {
                        return Err(Message::undefined_variable(node.location, *id));
                    };

                    if let Type::Ref { mutability, .. } = &var_data.var_type {
                        if mutability.is_const() && does_mutate {
                            return Err(Message::const_ref_mutation(node.location, *id));
                        }
                    } else if var_data.mutability.is_const() && does_mutate {
                        return Err(Message::const_var_mutation(node.location, *id));
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

            if lhs_type.ty != rhs_type.ty {
                self.error(Message::from_string(
                    rhs.location(),
                    format!(
                        "Cannot perform operation on objects with different types: {lhs_type} and {rhs_type}",
                    ),
                ));
            }

            if lhs_type.mutability.is_const() && does_mutate {
                return Err(Message::const_mutation(
                    lhs.location(),
                    &lhs_type.ty.as_str(),
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
        if ids.is_empty() {
            unreachable!();
        }

        let mut iter = ids.iter().peekable();

        let Some(entry) = self.table.lookup(*iter.next().unwrap()) else {
            return Err(Message::undefined_id(rhs.location(), *ids.last().unwrap()));
        };

        let entry = entry.clone();

        match &entry.kind {
            SymbolKind::VarDef(data) => self.check_member_access(entry.name, data, iter, rhs),
            _ => unimplemented!("Entry.kind: {:?}", entry.kind),
        }
    }

    pub fn analyze_indexing(&mut self, lhs: &mut Ast, index: &mut Ast) -> Result<(), Message> {
        lhs.accept_mut(self)?;
        index.accept_mut(self)?;

        let location = lhs.location();

        match lhs {
            Ast::Value(Value {
                kind: ValueKind::Identifier(var_name),
                ..
            }) => {
                let Some(var_entry) = self.table.lookup(*var_name) else {
                    return Err(Message::undefined_id(location, *var_name));
                };

                let SymbolKind::VarDef(var_data) = &var_entry.kind else {
                    return Err(Message::from_string(
                        location,
                        format!("{var_name} is not an variable"),
                    ));
                };

                let Type::Array { .. } = &var_data.var_type else {
                    return Err(Message::from_string(
                        location,
                        format!("{var_name} is not an array"),
                    ));
                };
            }
            _ => {
                return Err(Message::from_string(
                    lhs.location(),
                    format!("Can't index {}", lhs.name()),
                ));
            }
        }

        let index_ty = self.get_type(index);
        if !index_ty.ty.is_integer() {
            return Err(Message::from_string(
                index.location(),
                format!("Invalid index type: {}", index_ty.ty),
            ));
        }

        Ok(())
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

    fn check_members(
        &mut self,
        name: Ident,
        type_info: &TypeInfo,
        members: &mut Peekable<Iter<Ident>>,
        location: Location,
    ) -> Result<Type, Message> {
        let Some(next) = members.next() else {
            return Ok(type_info.ty.clone());
        };

        if let Some(member) = type_info.members.get(next) {
            let member_type = self.table.lookup_type(&member.ty).unwrap();

            self.check_members(*next, &member_type, members, location)
        } else {
            Err(Message::from_string(
                location,
                format!("\"{name}\" doesn't have member named \"{next}\""),
            ))
        }
    }

    fn check_member_access(
        &mut self,
        var_name: Ident,
        var_data: &VarDefData,
        mut members: Peekable<Iter<Ident>>,
        node: &mut Ast,
    ) -> Result<(), Message> {
        let location = node.location();
        let var_type = &var_data.var_type;

        let member_type = if let Some(type_info) = self.table.lookup_type(var_type) {
            self.check_members(var_name, &type_info, &mut members, location)?
        } else {
            unreachable!()
        };

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
                if operation.does_mutate() && var_data.mutability.is_const() {
                    return Err(Message::const_var_mutation(location, var_name));
                }

                let Ast::Value(Value {
                    kind: ValueKind::Identifier(_),
                    ..
                }) = lhs.as_ref()
                else {
                    return Err(Message::unreachable(
                        location,
                        format!("Expected member access, actually: {lhs:?}"),
                    ));
                };

                rhs.accept_mut(self)?;

                let rhs_type = self.get_type(rhs);

                if member_type != rhs_type.ty {
                    return Err(Message::from_string(
                                rhs.location(),
                                format!(
                                    "Cannot perform operation on objects with different types: {member_type} and {rhs_type}",
                                ),
                            ));
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
        let loc = lhs.location();

        let lhs_id = match lhs {
            Ast::Value(Value {
                kind: ValueKind::Identifier(id),
                ..
            }) => *id,
            _ => Ident::default(),
        };

        ids.push(lhs_id);

        match rhs {
            Ast::Expression(Expression {
                kind: ExpressionKind::Get { lhs, rhs },
                ..
            }) => Self::preprocess_access_tree(ids, lhs, rhs),
            Ast::Expression(Expression {
                kind: ExpressionKind::Access { lhs, rhs },
                ..
            }) => Self::preprocess_access_tree(ids, lhs, rhs),
            Ast::Expression(Expression {
                kind:
                    ExpressionKind::Binary {
                        operation: BinaryOperation::Assign,
                        lhs: last_lhs,
                        ..
                    },
                ..
            }) => {
                match last_lhs.as_ref() {
                    Ast::Value(Value {
                        kind: ValueKind::Identifier(last_lhs_id),
                        ..
                    }) => {
                        ids.push(*last_lhs_id);
                    }
                    Ast::Value(Value {
                        kind:
                            ValueKind::Call {
                                identifier: last_lhs_id,
                                ..
                            },
                        ..
                    }) => {
                        ids.push(*last_lhs_id);
                    }
                    _ => {}
                }
                Ok(rhs)
            }
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
                    lhs_type.ty,
                    rhs_type.ty
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
