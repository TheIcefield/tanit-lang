use tanitc_ast::program_ctx::statement_ctx::{
    attributes_ctx::AttributesCtx, definition_ctx::const_def_ctx::ConstDefCtx,
};
use tanitc_attributes::{Mutability, Visibility};
use tanitc_hir::hir::definitions::variables::{VariableAttributes, VariableDef};
use tanitc_messages::Message;

use crate::{AstLowResult, AstLowering};

impl AstLowering {
    pub(crate) fn low_const_def_ctx(
        &mut self,
        const_def_ctx: &ConstDefCtx,
    ) -> AstLowResult<VariableDef> {
        let location = const_def_ctx.const_tkn.get_location();
        let attributes = self.low_const_def_attributes(&const_def_ctx.attributes_ctx)?;
        let var_type = self.low_type_ctx(&const_def_ctx.type_ctx.type_ctx)?.ty;
        let value = Some(Box::new(
            self.low_expression_ctx(&const_def_ctx.value_ctx.value_ctx)?,
        ));
        let identifier = self
            .low_name_ctx(&const_def_ctx.name_ctx)
            .get_id()
            .ok_or(Message::empty_name_spec(location))?;

        Ok(VariableDef {
            location,
            attributes,
            identifier,
            var_type,
            visibility: Visibility::Local,
            mutability: Mutability::Immutable,
            value,
        })
    }

    fn low_const_def_attributes(&self, ctx: &AttributesCtx) -> AstLowResult<VariableAttributes> {
        self.expect_incompatible_attribute(&ctx.safe_tkn)?;
        self.expect_incompatible_attribute(&ctx.unsafe_tkn)?;

        Ok(VariableAttributes {
            publicity: self.low_publicity_token(&ctx.pub_tkn),
        })
    }
}
