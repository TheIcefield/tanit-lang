use tanitc_ast::program_ctx::type_ctx::array_type_ctx::ArrayTypeCtx;
use tanitc_hir::hir::type_spec::{ArraySize, Type, TypeSpec};

use crate::{AstLowResult, AstLowering};

impl AstLowering {
    pub(crate) fn low_array_type_ctx(&self, type_ctx: &ArrayTypeCtx) -> AstLowResult<TypeSpec> {
        let location = type_ctx.lsb_tkn.get_location();

        let value_type = Box::new(self.low_type_ctx(&type_ctx.type_ctx)?.ty);

        let ty = Type::Array {
            size: ArraySize::Unknown,
            value_type,
        };

        Ok(TypeSpec { location, ty })
    }
}
