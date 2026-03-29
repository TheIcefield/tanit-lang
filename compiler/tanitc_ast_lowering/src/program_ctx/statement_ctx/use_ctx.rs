use tanitc_ast::program_ctx::statement_ctx::use_ctx::UseCtx;
use tanitc_hir::hir::uses::Use;

use crate::{AstLowResult, AstLowering};

impl AstLowering {
    pub(crate) fn low_use_ctx(&self, ctx: &UseCtx) -> AstLowResult<Use> {
        let location = ctx.use_tkn.get_location();
        let name = self.low_name_spec_ctx(&ctx.name_spec_ctx)?;

        Ok(Use { location, name })
    }
}
