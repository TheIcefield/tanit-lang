use tanitc_ast::program_ctx::statement_ctx::{StatementCtx, StatementsCtx};
use tanitc_hir::hir::Hir;

use crate::{AstLowResult, AstLowering};

pub(crate) mod attributes_ctx;
pub(crate) mod block_ctx;
pub(crate) mod branch_ctx;
pub(crate) mod control_flow_ctx;
pub(crate) mod definition_ctx;
pub(crate) mod expression_ctx;
pub(crate) mod use_ctx;

impl AstLowering {
    pub(crate) fn low_statements_ctx(
        &mut self,
        statements_ctx: &StatementsCtx,
    ) -> AstLowResult<Vec<Hir>> {
        let mut statements_hir = Vec::<Hir>::with_capacity(statements_ctx.statements.len());

        for (statements_ctx, _) in statements_ctx.statements.iter() {
            let Some(statement_ctx) = statements_ctx else {
                continue;
            };

            match self.low_statement_ctx(statement_ctx) {
                Err(err) => self.error(err),
                Ok(stmt) => statements_hir.push(stmt),
            }
        }

        statements_hir.shrink_to_fit();
        Ok(statements_hir)
    }

    pub(crate) fn low_statement_ctx(&mut self, statement_ctx: &StatementCtx) -> AstLowResult<Hir> {
        match statement_ctx {
            StatementCtx::Definition(ctx) => self.low_definition_ctx(ctx).map(Hir::Definition),
            StatementCtx::Block(ctx) => self.low_block_ctx(ctx).map(Hir::Block),
            StatementCtx::Expression(ctx) => self.low_expression_ctx(ctx).map(Hir::Expression),
            StatementCtx::ControlFlow(ctx) => self.low_control_flow_ctx(ctx).map(Hir::ControlFlow),
            StatementCtx::Branch(ctx) => self.low_branch_ctx(ctx).map(Hir::BranchStmt),
            StatementCtx::Use(ctx) => self.low_use_ctx(ctx).map(Hir::Use),
        }
    }
}
