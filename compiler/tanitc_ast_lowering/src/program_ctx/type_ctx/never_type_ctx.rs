use tanitc_ast::program_ctx::type_ctx::never_type_ctx::NeverTypeCtx;
use tanitc_hir::hir::type_spec::{Type, TypeSpec};

use crate::{AstLowResult, AstLowering};

impl AstLowering {
    pub(crate) fn low_never_type_ctx(&self, type_ctx: &NeverTypeCtx) -> AstLowResult<TypeSpec> {
        let location = type_ctx.excm_tkn.get_location();
        let ty = Type::Never;

        Ok(TypeSpec { location, ty })
    }
}
