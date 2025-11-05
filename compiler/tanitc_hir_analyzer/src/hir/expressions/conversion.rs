use tanitc_attributes::Mutability;
use tanitc_hir::hir::expressions::conversion::ConversionExpr;

use crate::{symbol_table::type_info::TypeInfo, AnalyzeResult, Analyzer};

impl Analyzer {
    pub(crate) fn analyze_conversion_expr(
        &mut self,
        _expr: &mut ConversionExpr,
    ) -> AnalyzeResult<()> {
        todo!()
    }

    pub(crate) fn get_conversion_expr_type(&self, expr: &ConversionExpr) -> TypeInfo {
        TypeInfo {
            ty: expr.ty.get_type(),
            mutability: Mutability::Mutable,
            ..Default::default()
        }
    }
}
