use tanitc_ast::program_ctx::statement_ctx::expression_ctx::unary_ctx::{UnaryCtx, UnaryOpCtx};
use tanitc_hir::hir::expressions::unary::{UnaryExpr, UnaryOperation};

use crate::{AstLowResult, AstLowering};

impl AstLowering {
    pub(crate) fn low_unary_expression_ctx(&mut self, ctx: &UnaryCtx) -> AstLowResult<UnaryExpr> {
        let operation = self.low_unary_operator(&ctx.unary_op_ctx);
        let node = Box::new(self.low_expression_ctx(&ctx.expression_ctx)?);
        let location = node.location();

        Ok(UnaryExpr {
            location,
            operation,
            node,
        })
    }

    fn low_unary_operator(&self, op: &UnaryOpCtx) -> UnaryOperation {
        match op {
            UnaryOpCtx::Add(_) => UnaryOperation::Add,
            UnaryOpCtx::Sub(_) => UnaryOperation::Sub,
            UnaryOpCtx::Ref(_, None) => UnaryOperation::Ref,
            UnaryOpCtx::Ref(_, Some(_)) => UnaryOperation::RefMut,
        }
    }
}
