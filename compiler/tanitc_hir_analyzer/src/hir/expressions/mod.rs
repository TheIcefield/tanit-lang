use tanitc_attributes::Mutability;
use tanitc_hir::hir::{
    expressions::{variable::Variable, Expression},
    types::Type,
};
use tanitc_messages::Message;

use crate::{
    symbol_table::{entry::SymbolKind, type_info::TypeInfo},
    AnalyzeResult, Analyzer,
};

pub(crate) mod binary;
pub(crate) mod call_expr;
pub(crate) mod conversion;
pub(crate) mod indexing;
pub(crate) mod literal;
pub(crate) mod unary;

impl Analyzer {
    pub(crate) fn analyze_expression(&mut self, expr: &mut Expression) -> AnalyzeResult<()> {
        match expr {
            Expression::Binary(expr) => self.analyze_binary_expr(expr),
            Expression::Unary(expr) => self.analyze_unary_expr(expr),
            Expression::Conversion(expr) => self.analyze_conversion_expr(expr),
            Expression::Indexing(expr) => self.analyze_indexing_expr(expr),
            Expression::Call(call_expr) => self.analyze_call_expr(call_expr),
            Expression::Variable(var) => self.analyze_variable_usage(var),
            Expression::Literal(lit) => self.analyze_literal(lit),
        }
    }

    pub(crate) fn get_expr_type(&self, expr: &Expression) -> TypeInfo {
        match expr {
            Expression::Binary(expr) => self.get_binary_expr_type(expr),
            Expression::Unary(expr) => self.get_unary_expr_type(expr),
            Expression::Conversion(expr) => self.get_conversion_expr_type(expr),
            Expression::Indexing(expr) => self.get_indexing_expr_type(expr),
            Expression::Call(call_expr) => self.get_call_expr_type(call_expr),
            Expression::Variable(var) => self.get_variable_type(var),
            Expression::Literal(lit) => self.get_literal_type(lit),
        }
    }

    fn analyze_variable_usage(&mut self, var: &mut Variable) -> AnalyzeResult<()> {
        if self.has_symbol(var.id) {
            Ok(())
        } else {
            Err(Message::undefined_id(var.location, var.id))
        }
    }

    fn get_variable_type(&self, var: &Variable) -> TypeInfo {
        let mut type_info = TypeInfo {
            ty: Type::new(),
            mutability: Mutability::Mutable,
            ..Default::default()
        };

        // Search entries with name id
        let Some(entry) = self.table.lookup(var.id) else {
            return type_info;
        };

        match &entry.kind {
            SymbolKind::VarDef(data) => {
                type_info.ty = data.var_type.clone();
                type_info.mutability = data.mutability;

                let Some(found_type_info) = self.table.lookup_type(&data.var_type) else {
                    return type_info;
                };

                type_info = found_type_info;
                type_info.mutability = data.mutability;

                type_info
            }
            SymbolKind::FuncDef(data) => {
                type_info.ty = Type::Func(data.ty.clone());
                type_info
            }
            _ => type_info,
        }
    }
}

pub(crate) fn get_ordinal_number_suffix(num: usize) -> &'static str {
    match num % 10 {
        0 => "st",
        1 => "nd",
        2 => "rd",
        _ => "th",
    }
}
