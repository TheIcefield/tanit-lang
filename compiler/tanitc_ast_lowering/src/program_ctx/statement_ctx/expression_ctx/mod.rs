use tanitc_ast::program_ctx::{
    name_ctx::NameSpecCtx, statement_ctx::expression_ctx::ExpressionCtx,
};
use tanitc_hir::hir::expressions::{variable::Variable, Expression};

use crate::{AstLowResult, AstLowering};

pub(crate) mod binary_ctx;
pub(crate) mod call_ctx;
pub(crate) mod conversion_ctx;
pub(crate) mod indexing_ctx;
pub(crate) mod literal_ctx;
pub(crate) mod unary_ctx;

impl AstLowering {
    pub(crate) fn low_expression_ctx(&mut self, ctx: &ExpressionCtx) -> AstLowResult<Expression> {
        match ctx {
            ExpressionCtx::ParenCtx(ctx) => self.low_expression_ctx(&ctx.expression_ctx),
            ExpressionCtx::Literal(ctx) => self.low_literal_ctx(ctx).map(Expression::Literal),
            ExpressionCtx::Unary(ctx) => self.low_unary_expression_ctx(ctx).map(Expression::Unary),
            ExpressionCtx::Binary(ctx) => {
                self.low_binary_expression_ctx(ctx).map(Expression::Binary)
            }
            ExpressionCtx::Conversion(ctx) => self
                .low_conversion_expression_ctx(ctx)
                .map(Expression::Conversion),
            ExpressionCtx::Indexing(ctx) => self
                .low_indexing_expression_ctx(ctx)
                .map(Expression::Indexing),
            ExpressionCtx::Call(ctx) => self.low_call_ctx(ctx).map(Expression::Call),
            ExpressionCtx::Variable(ctx) => self.low_variable_ctx(ctx).map(Expression::Variable),
        }
    }

    fn low_variable_ctx(&self, var_name_spec_ctx: &NameSpecCtx) -> AstLowResult<Variable> {
        let name = self.low_name_spec_ctx(var_name_spec_ctx)?;
        let location = name.location;

        Ok(Variable { location, name })
    }
}
