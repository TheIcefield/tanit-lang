use tanitc_ast::program_ctx::type_ctx::array_type_ctx::{ArrayTypeCtx, ArrayTypeLengthCtx};
use tanitc_hir::hir::{
    expressions::Expression,
    type_spec::{ArrayType, SliceType, Type, TypeSpec},
};

use crate::{AstLowResult, AstLowering};

impl AstLowering {
    pub(crate) fn low_array_type_ctx(&mut self, type_ctx: &ArrayTypeCtx) -> AstLowResult<TypeSpec> {
        let location = type_ctx.lsb_tkn.get_location();

        let internal_type = Box::new(self.low_type_ctx(&type_ctx.type_ctx)?.ty);

        let ty = if let Some(length) = &type_ctx.length_ctx {
            let size = self.low_array_type_length_ctx(&length)?;

            Type::Array(ArrayType {
                internal_type,
                size,
            })
        } else {
            Type::Slice(SliceType { internal_type })
        };

        Ok(TypeSpec { location, ty })
    }

    fn low_array_type_length_ctx(
        &mut self,
        length_ctx: &ArrayTypeLengthCtx,
    ) -> AstLowResult<Box<Expression>> {
        Ok(Box::new(
            self.low_expression_ctx(&length_ctx.expression_ctx)?,
        ))
    }
}
