use tanitc_ast::program_ctx::statement_ctx::expression_ctx::call_ctx::{
    CallCtx, CallParamCtx, CallParamsCtx, NamedCallParamCtx, PositionalCallParamCtx,
};
use tanitc_hir::hir::expressions::call::{CallArg, CallExpr, NamedCallArg, PositionalCallArg};
use tanitc_messages::Message;

use crate::{AstLowResult, AstLowering};

impl AstLowering {
    pub(crate) fn low_call_ctx(&mut self, ctx: &CallCtx) -> AstLowResult<CallExpr> {
        let expr = Box::new(self.low_expression_ctx(&ctx.expression_ctx)?);
        let location = expr.location();
        let arguments = self.low_call_params_ctx(&ctx.params_ctx)?;

        Ok(CallExpr {
            location,
            expr,
            arguments,
        })
    }

    fn low_call_params_ctx(&mut self, ctx: &CallParamsCtx) -> AstLowResult<Vec<CallArg>> {
        let mut arguments = Vec::<CallArg>::new();

        for (param_index, (param_ctx, _)) in ctx.params.iter().enumerate() {
            let Some(param_ctx) = param_ctx else {
                continue;
            };

            match self.low_call_param_ctx(param_ctx, param_index) {
                Err(err) => self.error(err),
                Ok(param) => arguments.push(param),
            }
        }

        Ok(arguments)
    }

    fn low_call_param_ctx(&mut self, ctx: &CallParamCtx, index: usize) -> AstLowResult<CallArg> {
        match ctx {
            CallParamCtx::Named(ctx) => self.low_named_call_param_ctx(ctx).map(CallArg::Notified),
            CallParamCtx::Positional(ctx) => self
                .low_positional_call_param_ctx(ctx, index)
                .map(CallArg::Positional),
        }
    }

    fn low_named_call_param_ctx(&mut self, ctx: &NamedCallParamCtx) -> AstLowResult<NamedCallArg> {
        let expr = Box::new(self.low_expression_ctx(&ctx.expression_ctx)?);
        let location = expr.location();
        let id = self
            .low_name_ctx(&ctx.name_ctx)
            .get_id()
            .ok_or(Message::empty_name_spec(location))?;

        Ok(NamedCallArg { location, id, expr })
    }

    fn low_positional_call_param_ctx(
        &mut self,
        ctx: &PositionalCallParamCtx,
        id: usize,
    ) -> AstLowResult<PositionalCallArg> {
        let expr = Box::new(self.low_expression_ctx(&ctx.expression_ctx)?);
        let location = expr.location();

        Ok(PositionalCallArg { location, id, expr })
    }
}
