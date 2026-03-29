use tanitc_attributes::Mutability;
use tanitc_hir::hir::{
    expressions::{
        binary::{BinaryExpr, BinaryOperation},
        unary::{UnaryExpr, UnaryOperation},
        Expression,
    },
    type_spec::Type,
};
use tanitc_messages::Message;

use crate::{
    symbol_table::{entry::SymbolKind, type_info::TypeInfo},
    AnalyzeResult, Analyzer,
};

impl Analyzer {
    pub(crate) fn analyze_binary_expr(&mut self, expr: &mut BinaryExpr) -> AnalyzeResult<()> {
        self.analyze_expression(&mut expr.rhs)?;

        let rhs_type = self.get_expr_type(&expr.rhs);
        let does_mutate = expr.operation.does_mutate();

        let lhs_type = match expr.lhs.as_mut() {
            Expression::Variable(var) => {
                let entry = self
                    .table
                    .lookup_name_spec(&var.name)
                    .map_err(|err| Message::new(var.location, err))?;

                let SymbolKind::VarDef(var_data) = &entry.kind else {
                    return Err(Message::undefined_variable(var.location, &var.name));
                };

                if var_data.mutability.is_const() && does_mutate {
                    return Err(Message::const_var_mutation(var.location, &var.name));
                }

                if let Type::Ref(ref_type) = &var_data.var_type {
                    if ref_type.mutability.is_const() && does_mutate {
                        return Err(Message::const_ref_mutation(var.location, &var.name));
                    }
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
                return Err(Message::new(
                    lit.location(),
                    format!(
                        "Cannot perform operation with {} in this context",
                        lit.kind_str()
                    ),
                ))
            }
            expr => {
                return Err(Message::new(
                    expr.location(),
                    format!(
                        "Cannot perform operation with {} in this context",
                        expr.kind_str()
                    ),
                ))
            }
        };

        if lhs_type != rhs_type.ty {
            self.error(Message::new(
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
}
