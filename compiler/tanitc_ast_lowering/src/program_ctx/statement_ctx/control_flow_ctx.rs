use tanitc_ast::program_ctx::statement_ctx::control_flow_ctx::{
    break_ctx::BreakCtx, continue_ctx::ContinueCtx, return_ctx::ReturnCtx, ControlFlowCtx,
};
use tanitc_hir::hir::control_flows::{ControlFlow, ControlFlowKind};

use crate::{AstLowResult, AstLowering};

impl AstLowering {
    pub(crate) fn low_control_flow_ctx(
        &mut self,
        ctx: &ControlFlowCtx,
    ) -> AstLowResult<ControlFlow> {
        match ctx {
            ControlFlowCtx::Break(ctx) => self.low_break_ctx(ctx),
            ControlFlowCtx::Continue(ctx) => self.low_continue_ctx(ctx),
            ControlFlowCtx::Return(ctx) => self.low_return_ctx(ctx),
        }
    }

    fn low_break_ctx(&self, ctx: &BreakCtx) -> AstLowResult<ControlFlow> {
        let location = ctx.break_tkn.get_location();

        let ret = None;
        let kind = ControlFlowKind::Break { ret };

        Ok(ControlFlow { location, kind })
    }

    fn low_continue_ctx(&self, ctx: &ContinueCtx) -> AstLowResult<ControlFlow> {
        let location = ctx.continue_tkn.get_location();
        let kind = ControlFlowKind::Continue;

        Ok(ControlFlow { location, kind })
    }

    fn low_return_ctx(&mut self, ctx: &ReturnCtx) -> AstLowResult<ControlFlow> {
        let location = ctx.return_tkn.get_location();

        let ret = if let Some(ret_expr_ctx) = &ctx.return_expression_ctx {
            Some(Box::new(self.low_expression_ctx(ret_expr_ctx)?))
        } else {
            None
        };

        let kind = ControlFlowKind::Return { ret };

        Ok(ControlFlow { location, kind })
    }
}
