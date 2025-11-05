use tanitc_ast::program_ctx::statement_ctx::branch_ctx::{
    else_ctx::{ElseBodyCtx, ElseCtx},
    if_ctx::IfCtx,
    loop_ctx::LoopCtx,
    while_ctx::WhileCtx,
    BranchCtx,
};
use tanitc_hir::hir::branches::{Branch, Else, ElseBody, If, Loop, While};

use crate::{AstLowResult, AstLowering};

impl AstLowering {
    pub(crate) fn low_branch_ctx(&mut self, ctx: &BranchCtx) -> AstLowResult<Branch> {
        match ctx {
            BranchCtx::Loop(ctx) => self.low_loop_ctx(ctx).map(Branch::Loop),
            BranchCtx::While(ctx) => self.low_while_ctx(ctx).map(Branch::While),
            BranchCtx::If(ctx) => self.low_if_ctx(ctx).map(Branch::If),
            BranchCtx::Else(ctx) => self.low_else_ctx(ctx).map(Branch::Else),
        }
    }

    fn low_loop_ctx(&mut self, ctx: &LoopCtx) -> AstLowResult<Loop> {
        let location = ctx.loop_tkn.get_location();
        let body = Box::new(self.low_block_ctx(&ctx.block_ctx)?);

        Ok(Loop { location, body })
    }

    fn low_while_ctx(&mut self, ctx: &WhileCtx) -> AstLowResult<While> {
        let location = ctx.while_tkn.get_location();
        let body = Box::new(self.low_block_ctx(&ctx.block_ctx)?);
        let condition = Box::new(self.low_expression_ctx(&ctx.expression_ctx)?);

        Ok(While {
            location,
            body,
            condition,
        })
    }

    fn low_if_ctx(&mut self, ctx: &IfCtx) -> AstLowResult<If> {
        let location = ctx.if_tkn.get_location();
        let body = Box::new(self.low_block_ctx(&ctx.block_ctx)?);
        let condition = Box::new(self.low_expression_ctx(&ctx.expression_ctx)?);

        Ok(If {
            location,
            body,
            condition,
        })
    }

    fn low_else_ctx(&mut self, ctx: &ElseCtx) -> AstLowResult<Else> {
        let location = ctx.else_tkn.get_location();

        let body = match &ctx.body_ctx {
            ElseBodyCtx::If(ctx) => self
                .low_if_ctx(ctx)
                .map(|ctx| ElseBody::If(Box::new(ctx)))?,
            ElseBodyCtx::Block(block) => self
                .low_block_ctx(block)
                .map(|ctx| ElseBody::Block(Box::new(ctx)))?,
        };

        Ok(Else { location, body })
    }
}
