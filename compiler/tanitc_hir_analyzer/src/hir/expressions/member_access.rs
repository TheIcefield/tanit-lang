use tanitc_hir::hir::expressions::member_access::MemberAccessExpr;

use crate::{symbol_table::type_info::TypeInfo, AnalyzeResult, Analyzer};

impl Analyzer {
    pub(crate) fn analyze_member_access_expr(&self, _expr: &MemberAccessExpr) -> AnalyzeResult<()> {
        Ok(())
    }

    pub(crate) fn get_member_access_expr_type(&self, _expr: &MemberAccessExpr) -> TypeInfo {
        TypeInfo::default()
    }
}
