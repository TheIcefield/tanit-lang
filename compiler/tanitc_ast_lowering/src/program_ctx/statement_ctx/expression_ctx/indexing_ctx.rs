use tanitc_ast::program_ctx::statement_ctx::expression_ctx::indexing_ctx::IndexingCtx;
use tanitc_hir::hir::expressions::indexing::IndexingExpr;

use crate::{AstLowResult, AstLowering};

impl AstLowering {
    pub(crate) fn low_indexing_expression_ctx(
        &mut self,
        ctx: &IndexingCtx,
    ) -> AstLowResult<IndexingExpr> {
        let lhs = Box::new(self.low_expression_ctx(&ctx.expression_ctx)?);
        let index = Box::new(self.low_expression_ctx(&ctx.index_ctx.expression_ctx)?);
        let location = lhs.location();

        Ok(IndexingExpr {
            location,
            lhs,
            index,
        })
    }
}
