use std::{iter::Peekable, slice::Iter};

use tanitc_attributes::{Mutability, Safety};
use tanitc_hir::hir::{
    expressions::{
        binary::{BinaryExpr, BinaryOperation},
        unary::{UnaryExpr, UnaryOperation},
        variable::Variable,
        Expression,
    },
    types::Type,
};
use tanitc_ident::Ident;
use tanitc_lexer::location::Location;
use tanitc_messages::Message;

use crate::{
    symbol_table::{
        entry::{SymbolKind, VarDefData},
        type_info::TypeInfo,
    },
    AnalyzeResult, Analyzer,
};

impl Analyzer {
    pub(crate) fn analyze_binary_expr(&mut self, expr: &mut BinaryExpr) -> AnalyzeResult<()> {
        if expr.operation == BinaryOperation::Access {
            return self.analyze_member_access(expr);
        }

        if expr.operation == BinaryOperation::ScopeRes {
            return self.analyze_scope_res_expr(expr);
        }

        self.analyze_expression(&mut expr.rhs)?;

        let rhs_type = self.get_expr_type(&expr.rhs);
        let does_mutate = expr.operation.does_mutate();

        let lhs_type = match expr.lhs.as_mut() {
            Expression::Variable(var) => {
                let Some(entry) = self.table.lookup(var.id) else {
                    return Err(Message::undefined_id(var.location, var.id));
                };

                let SymbolKind::VarDef(var_data) = &entry.kind else {
                    return Err(Message::undefined_variable(var.location, var.id));
                };

                if let Type::Ref(ref_type) = &var_data.var_type {
                    if ref_type.mutability.is_const() && does_mutate {
                        return Err(Message::const_ref_mutation(var.location, var.id));
                    }
                } else if var_data.mutability.is_const() && does_mutate {
                    return Err(Message::const_var_mutation(var.location, var.id));
                }

                var_data.var_type.clone()
            }
            Expression::Unary(UnaryExpr {
                node,
                operation,
                location,
            }) => {
                if !matches!(operation, UnaryOperation::Deref) {
                    return Err(Message::new(
                        *location,
                        "Cannot perform operation on rvalue",
                    ));
                }
                self.get_expr_type(node.as_ref()).ty
            }
            Expression::Literal(lit) => {
                return Err(Message::from_string(
                    lit.location(),
                    format!(
                        "Cannot perform operation with {} in this context",
                        lit.kind_str()
                    ),
                ))
            }
            expr => {
                return Err(Message::from_string(
                    expr.location(),
                    format!(
                        "Cannot perform operation with {} in this context",
                        expr.kind_str()
                    ),
                ))
            }
        };

        if lhs_type != rhs_type.ty {
            self.error(Message::from_string(
                expr.rhs.location(),
                format!(
                    "Cannot perform operation on objects with different types: {} and {rhs_type}",
                    lhs_type
                ),
            ));
        }

        Ok(())
    }

    pub(crate) fn get_binary_expr_type(&self, expr: &BinaryExpr) -> TypeInfo {
        match expr.operation {
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

            _ => self.get_expr_type(&expr.lhs),
        }
    }

    pub(crate) fn analyze_scope_res_expr(
        &mut self,
        scope_res_expr: &mut BinaryExpr,
    ) -> AnalyzeResult<()> {
        let mut ids = Vec::<Ident>::new();

        Self::preprocess_access_tree(&mut ids, scope_res_expr)?;
        if ids.is_empty() {
            return Err(Message::unreachable(
                scope_res_expr.location,
                format!("<analyze_scope_res_expr>: IDS is empty"),
            ));
        }

        let Some(entry) = self.table.lookup_qualified(ids.iter().peekable()) else {
            return Err(Message::undefined_id(
                scope_res_expr.location,
                *ids.last().unwrap(),
            ));
        };

        let kind = entry.kind.clone();

        match &kind {
            SymbolKind::Enum(data) => self.access_enum(data, &mut scope_res_expr.rhs),
            SymbolKind::Variant(data) => self.access_variant(data, &mut scope_res_expr.rhs),
            SymbolKind::StructDef(data) => self.access_struct_def(data, &mut scope_res_expr.rhs),
            SymbolKind::UnionDef(data) => self.access_union_def(data, &mut scope_res_expr.rhs),
            SymbolKind::FuncDef(_) => self.access_func_def(&mut scope_res_expr.rhs),
            _ => Err(Message::from_string(
                scope_res_expr.location,
                format!("Unaccessible: {kind:?}"),
            )),
        }
    }

    pub(crate) fn analyze_member_access(
        &mut self,
        member_access_expr: &mut BinaryExpr,
    ) -> AnalyzeResult<()> {
        let mut ids = Vec::<Ident>::new();

        Self::preprocess_access_tree(&mut ids, member_access_expr)?;
        if ids.is_empty() {
            return Err(Message::unreachable(
                member_access_expr.location,
                format!("<analyze_member_access>: IDS is empty"),
            ));
        }

        let mut iter = ids.iter().peekable();

        let Some(entry) = self.table.lookup(*iter.next().unwrap()) else {
            return Err(Message::undefined_id(
                member_access_expr.rhs.location(),
                *ids.last().unwrap(),
            ));
        };

        let kind = entry.kind.clone();
        Ok(())

        // match &kind {
        //     SymbolKind::VarDef(data) => self.check_member_access(entry.name, data, iter, rhs),
        //     _ => unimplemented!("Entry.kind: {kind:?}"),
        // }
    }

    fn check_members(
        &self,
        name: Ident,
        type_info: &TypeInfo,
        members: &mut Peekable<Iter<Ident>>,
        location: Location,
    ) -> AnalyzeResult<Type> {
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

    /*

    fn check_member_access(
        &mut self,
        var_name: Ident,
        var_data: &VarDefData,
        mut members: Peekable<Iter<Ident>>,
        node: &mut Expression,
    ) -> AnalyzeResult<()> {
        let location = node.location();
        let var_type = &var_data.var_type;

        let member_type = if let Some(type_info) = self.table.lookup_type(var_type) {
            self.check_members(var_name, &type_info, &mut members, location)?
        } else {
            unreachable!("Can't find {var_type}")
        };

        match node {
            Expression::Binary(BinaryExpr {
                operation,
                lhs,
                rhs,
                ..
            }) => {
                if operation.does_mutate() && var_data.mutability.is_const() {
                    return Err(Message::const_var_mutation(location, var_name));
                }

                let Expression::Variable(_) = lhs.as_ref() else {
                    return Err(Message::unreachable(
                        location,
                        format!("Expected member access, actually: {lhs:?}"),
                    ));
                };

                self.analyze_expression(rhs)?;
                let rhs_type = self.get_expr_type(rhs);

                if member_type != rhs_type.ty {
                    return Err(Message::from_string(
                                rhs.location(),
                                format!(
                                    "Cannot perform operation on objects with different types: {member_type} and {rhs_type}",
                                ),
                            ));
                }
            }
            Expression::Variable(_) => {}
            _ => todo!("{node:?}"),
        }

        Ok(())
    }
     */

    fn preprocess_scope_resolution_tree(
        ids: &mut Vec<Ident>,
        expr: &BinaryExpr,
    ) -> AnalyzeResult<()> {
        let lhs_id = match expr.lhs.as_ref() {
            Expression::Variable(Variable { id, .. }) => *id,
            _ => Ident::default(),
        };

        ids.push(lhs_id);

        match expr.rhs.as_ref() {
            Expression::Binary(bin_expr) => {
                if bin_expr.operation == BinaryOperation::ScopeRes {
                    if let Expression::Variable(Variable {
                        id: last_lhs_id, ..
                    }) = expr.lhs.as_ref()
                    {
                        ids.push(*last_lhs_id);
                    }
                } else if bin_expr.operation == BinaryOperation::Access {
                    return Self::preprocess_scope_resolution_tree(ids, bin_expr);
                }
            }
            Expression::Variable(Variable { id: rhs_id, .. }) => {
                ids.push(*rhs_id);
            }
            _ => {
                return Err(Message::unreachable(
                    expr.location,
                    format!(
                        "Expected ScopeRes or Variable, actually: {}",
                        expr.rhs.kind_str()
                    ),
                ))
            }
        }

        Ok(())
    }

    fn preprocess_access_tree(ids: &mut Vec<Ident>, expr: &BinaryExpr) -> AnalyzeResult<()> {
        let lhs_id = match expr.lhs.as_ref() {
            Expression::Variable(Variable { id, .. }) => *id,
            _ => Ident::default(),
        };

        ids.push(lhs_id);

        match expr.rhs.as_ref() {
            Expression::Binary(bin_expr) => {
                if bin_expr.operation == BinaryOperation::Assign {
                    if let Expression::Variable(Variable {
                        id: last_lhs_id, ..
                    }) = expr.lhs.as_ref()
                    {
                        ids.push(*last_lhs_id);
                    }
                } else if bin_expr.operation == BinaryOperation::Access {
                    return Self::preprocess_access_tree(ids, bin_expr);
                }
            }
            Expression::Variable(Variable { id: rhs_id, .. }) => {
                ids.push(*rhs_id);
            }
            _ => {
                return Err(Message::unreachable(
                    expr.location,
                    format!(
                        "Expected Access or Variable, actually: {}",
                        expr.rhs.kind_str()
                    ),
                ))
            }
        }

        Ok(())
    }
}
