use tanitc_ast::program_ctx::statement_ctx::{
    attributes_ctx::AttributesCtx, definition_ctx::static_def_ctx::StaticDefCtx,
};
use tanitc_attributes::Visibility;
use tanitc_hir::hir::definitions::variables::{VariableAttributes, VariableDef};

use crate::{AstLowResult, AstLowering};

impl AstLowering {
    pub(crate) fn low_static_def_ctx(
        &mut self,
        static_def_ctx: &StaticDefCtx,
    ) -> AstLowResult<VariableDef> {
        let location = static_def_ctx.static_tkn.get_location();
        let attributes = self.low_static_def_attributes(&static_def_ctx.attributes_ctx)?;
        let identifier = self.low_name_ctx(&static_def_ctx.name_ctx).id;
        let var_type = self.low_type_ctx(&static_def_ctx.type_ctx.type_ctx)?.ty;
        let mutability = self.low_mut_token(&static_def_ctx.mut_tkn);
        let visibility = Visibility::Global;
        let value = Some(Box::new(
            self.low_expression_ctx(&static_def_ctx.value_ctx.value_ctx)?,
        ));

        Ok(VariableDef {
            location,
            attributes,
            identifier,
            var_type,
            visibility,
            mutability,
            value,
        })
    }

    fn low_static_def_attributes(&self, ctx: &AttributesCtx) -> AstLowResult<VariableAttributes> {
        self.expect_incompatible_attribute(&ctx.safe_tkn)?;
        self.expect_incompatible_attribute(&ctx.unsafe_tkn)?;

        Ok(VariableAttributes {
            publicity: self.low_publicity_token(&ctx.pub_tkn),
        })
    }
}
