use tanitc_ast::program_ctx::statement_ctx::{
    attributes_ctx::AttributesCtx, definition_ctx::alias_def_ctx::AliasDefCtx,
};
use tanitc_hir::hir::definitions::aliases::{AliasAttributes, AliasDef};

use crate::{AstLowResult, AstLowering};

impl AstLowering {
    pub(crate) fn low_alias_def_ctx(&self, alias_def_ctx: &AliasDefCtx) -> AstLowResult<AliasDef> {
        let location = alias_def_ctx.alias_tkn.get_location();
        let attributes = self.low_alias_def_attributes(&alias_def_ctx.attributes_ctx)?;
        let name = self.low_name_ctx(&alias_def_ctx.name_ctx);
        let value = self.low_type_ctx(&alias_def_ctx.type_ctx)?;

        Ok(AliasDef {
            location,
            attributes,
            name,
            value,
        })
    }

    fn low_alias_def_attributes(&self, ctx: &AttributesCtx) -> AstLowResult<AliasAttributes> {
        self.expect_incompatible_attribute(&ctx.safe_tkn)?;
        self.expect_incompatible_attribute(&ctx.unsafe_tkn)?;

        Ok(AliasAttributes {
            publicity: self.low_publicity_token(&ctx.pub_tkn),
        })
    }
}
