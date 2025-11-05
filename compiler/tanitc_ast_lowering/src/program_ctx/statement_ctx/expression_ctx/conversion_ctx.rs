use tanitc_ast::program_ctx::statement_ctx::expression_ctx::conversion_ctx::ConversionCtx;
use tanitc_hir::hir::expressions::conversion::ConversionExpr;

use crate::{AstLowResult, AstLowering};

impl AstLowering {
    pub(crate) fn low_conversion_expression_ctx(
        &mut self,
        ctx: &ConversionCtx,
    ) -> AstLowResult<ConversionExpr> {
        let ty = self.low_type_ctx(&ctx.type_ctx)?;
        let expr = Box::new(self.low_expression_ctx(&ctx.expression_ctx)?);
        let location = expr.location();

        Ok(ConversionExpr { location, expr, ty })
    }
}
