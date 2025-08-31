use std::{iter::Peekable, slice::Iter};

use tanitc_ast::{
    ast::expressions::{BinaryOperation, Expression, ExpressionKind, UnaryOperation},
    ast::{
        values::{CallArg, CallArgKind, Value, ValueKind},
        Ast,
    },
};
use tanitc_attributes::{Mutability, Safety};
use tanitc_ident::Ident;
use tanitc_lexer::location::Location;
use tanitc_messages::Message;
use tanitc_symbol_table::{
    entry::{SymbolKind, VarDefData},
    type_info::TypeInfo,
};
use tanitc_ty::Type;

use crate::Analyzer;

impl Analyzer {
    pub fn analyze_expression(&mut self, expr: &mut Expression) -> Result<(), Message> {
        let ret = match &mut expr.kind {
            ExpressionKind::Binary {
                operation,
                lhs,
                rhs,
            } => self.analyze_binary_expr(operation, lhs, rhs),
            ExpressionKind::Unary { operation, node } => self.analyze_unary_expr(operation, node),
            ExpressionKind::Access { lhs, rhs } => self.analyze_access_expr(lhs, rhs),
            ExpressionKind::Conversion { .. } => self.analyze_conversion_expr(),
            ExpressionKind::Get { lhs, rhs } => return self.analyze_member_access(lhs, rhs),
            ExpressionKind::Indexing { lhs, index } => return self.analyze_indexing(lhs, index),
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

    pub fn analyze_unary_expr(
        &mut self,
        operation: &UnaryOperation,
        node: &mut Box<Ast>,
    ) -> Result<Option<Expression>, Message> {
        node.accept_mut(self)?;
        let node_type = self.get_type(node);

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

        if node_type.ty.is_pointer()
            && *operation == UnaryOperation::Deref
            && self.get_current_safety() != Safety::Unsafe
        {
            return Err(Message::new(
                node.location(),
                "Dereferencing raw pointer require unsafe function or block",
            ));
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
            self.check_variable_def_types(node, Some(rhs))?;
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

    pub fn get_expr_type(&self, expr: &Expression) -> TypeInfo {
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
                | BinaryOperation::LogicalGe => TypeInfo {
                    ty: Type::Bool,
                    mutability: Mutability::Mutable,
                    ..Default::default()
                },

                _ => {
                    let lhs_type = self.get_type(lhs);

                    if let Type::Auto = lhs_type.ty {
                        return self.get_type(rhs);
                    }

                    lhs_type
                }
            },
            ExpressionKind::Unary { operation, node } => {
                let node_type = self.get_type(node);

                let (is_ref, mutability) = if *operation == UnaryOperation::Ref {
                    (true, Mutability::Immutable)
                } else if *operation == UnaryOperation::RefMut {
                    (true, Mutability::Mutable)
                } else {
                    (false, Mutability::Immutable)
                };

                if is_ref {
                    return TypeInfo {
                        ty: Type::Ref {
                            ref_to: Box::new(node_type.ty.clone()),
                            mutability,
                        },
                        mutability,
                        members: node_type.members,
                        ..Default::default()
                    };
                }

                node_type
            }
            ExpressionKind::Conversion { ty, .. } => TypeInfo {
                ty: ty.get_type(),
                mutability: Mutability::Mutable,
                ..Default::default()
            },
            ExpressionKind::Access { rhs, .. } => self.get_type(rhs),
            ExpressionKind::Get { rhs, .. } => self.get_type(rhs),
            ExpressionKind::Indexing { lhs, .. } => {
                let mut lhs_type = self.get_type(lhs);
                let Type::Array { ref value_type, .. } = &lhs_type.ty else {
                    unreachable!()
                };

                lhs_type.ty = value_type.as_ref().clone();
                lhs_type
            }
            ExpressionKind::Term { ty, .. } => TypeInfo {
                ty: ty.clone(),
                mutability: Mutability::Immutable,
                ..Default::default()
            },
        }
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

        if type_info.is_union && self.table.get_safety() != Safety::Unsafe {
            return Err(Message::new(
                location,
                "Access to union field is unsafe and requires an unsafe function or block",
            ));
        }

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
            Ast::Value(Value {
                kind: ValueKind::Identifier(_),
                ..
            }) => {}
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
        let Ast::Expression(node) = expr_node else {
            return Err(Message::from_string(
                expr_node.location(),
                format!("Expected Ast::Expression, actually: {}", expr_node.name()),
            ));
        };

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
    }
}

#[cfg(test)]
mod tests {
    use tanitc_ast::ast::{
        blocks::{Block, BlockAttributes},
        expressions::{BinaryOperation, Expression, ExpressionKind, UnaryOperation},
        functions::FunctionDef,
        types::TypeSpec,
        values::{Value, ValueKind},
        variables::VariableDef,
        Ast,
    };
    use tanitc_attributes::Safety;
    use tanitc_ident::Ident;
    use tanitc_lexer::location::Location;
    use tanitc_ty::Type;

    use crate::Analyzer;

    fn get_func_def(name: &str, statements: Vec<Ast>) -> FunctionDef {
        FunctionDef {
            identifier: Ident::from(name.to_string()),
            parameters: vec![],
            body: Some(Box::new(Block {
                statements,
                is_global: false,
                ..Default::default()
            })),
            ..Default::default()
        }
    }

    fn get_var_init(var_name: &str, ty: Type) -> Expression {
        Expression {
            location: Location::new(),
            kind: ExpressionKind::Binary {
                operation: BinaryOperation::Assign,
                lhs: Box::new(
                    VariableDef {
                        identifier: Ident::from(var_name.to_string()),
                        var_type: TypeSpec {
                            ty,
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                    .into(),
                ),
                rhs: Box::new(
                    Value {
                        location: Location::new(),
                        kind: ValueKind::Integer(0),
                    }
                    .into(),
                ),
            },
        }
    }

    fn get_raw_ptr_init(ptr_name: &str, var_name: &str) -> Expression {
        Expression {
            location: Location::new(),
            kind: ExpressionKind::Binary {
                operation: BinaryOperation::Assign,
                lhs: Box::new(
                    VariableDef {
                        identifier: Ident::from(ptr_name.to_string()),
                        var_type: TypeSpec {
                            ty: Type::Ptr(Box::new(Type::I32)),
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                    .into(),
                ),
                rhs: Box::new(
                    Expression {
                        location: Location::new(),
                        kind: ExpressionKind::Unary {
                            operation: UnaryOperation::Ref,
                            node: Box::new(
                                Value {
                                    location: Location::new(),
                                    kind: ValueKind::Identifier(Ident::from(var_name.to_string())),
                                }
                                .into(),
                            ),
                        },
                    }
                    .into(),
                ),
            },
        }
    }

    fn get_raw_ptr_deref(ptr_name: &str) -> Expression {
        Expression {
            location: Location::new(),
            kind: ExpressionKind::Unary {
                operation: UnaryOperation::Deref,
                node: Box::new(
                    Value {
                        location: Location::new(),
                        kind: ValueKind::Identifier(Ident::from(ptr_name.to_string())),
                    }
                    .into(),
                ),
            },
        }
    }

    #[test]
    fn check_deref_ptr_safety_good_test() {
        const MAIN_FUNC_NAME: &str = "main";
        const VAR_NAME: &str = "my_var";
        const PTR_NAME: &str = "my_ptr";

        let mut program = Ast::from(Block {
            is_global: true,
            statements: vec![get_func_def(
                MAIN_FUNC_NAME,
                vec![
                    get_var_init(VAR_NAME, Type::I32).into(),
                    get_raw_ptr_init(PTR_NAME, VAR_NAME).into(),
                    Block {
                        is_global: false,
                        attributes: BlockAttributes {
                            safety: Safety::Unsafe,
                        },
                        statements: vec![get_raw_ptr_deref(PTR_NAME).into()],
                        ..Default::default()
                    }
                    .into(),
                ],
            )
            .into()],
            ..Default::default()
        });

        let mut analyzer = Analyzer::new();
        program.accept_mut(&mut analyzer).unwrap();

        let errors = analyzer.get_errors();
        if !errors.is_empty() {
            panic!("{errors:#?}");
        }
    }

    #[test]
    fn check_deref_ptr_safety_bad_test() {
        const MAIN_FUNC_NAME: &str = "main";
        const VAR_NAME: &str = "my_var";
        const PTR_NAME: &str = "my_ptr";
        const EXPECTED_ERR: &str =
            "Semantic error: Dereferencing raw pointer require unsafe function or block";

        let mut program = Ast::from(Block {
            is_global: true,
            statements: vec![get_func_def(
                MAIN_FUNC_NAME,
                vec![
                    get_var_init(VAR_NAME, Type::I32).into(),
                    get_raw_ptr_init(PTR_NAME, VAR_NAME).into(),
                    get_raw_ptr_deref(PTR_NAME).into(),
                ],
            )
            .into()],
            ..Default::default()
        });

        let mut analyzer = Analyzer::new();
        program.accept_mut(&mut analyzer).unwrap();

        let errors = analyzer.get_errors();
        assert!(!errors.is_empty());
        assert_eq!(errors[0].text, EXPECTED_ERR);
    }
}
