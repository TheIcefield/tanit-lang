use tanitc_ast::program_ctx::statement_ctx::{
    attributes_ctx::AttributesCtx, definition_ctx::var_def_ctx::VarDefCtx,
};
use tanitc_attributes::Visibility;
use tanitc_hir::hir::{
    definitions::variables::{VariableAttributes, VariableDef},
    types::Type,
};

use crate::{AstLowResult, AstLowering};

impl AstLowering {
    pub(crate) fn low_variable_def_ctx(
        &mut self,
        var_def_ctx: &VarDefCtx,
    ) -> AstLowResult<VariableDef> {
        let location = var_def_ctx.var_tkn.get_location();
        let attributes = self.low_var_def_attributes(&var_def_ctx.attributes_ctx)?;
        let identifier = self.low_name_ctx(&var_def_ctx.name_ctx).id;
        let mutability = self.low_mut_token(&var_def_ctx.mut_tkn);
        let visibility = Visibility::Local;
        let var_type = if let Some(type_ctx) = &var_def_ctx.type_ctx {
            self.low_type_ctx(&type_ctx.type_ctx)?.ty
        } else {
            Type::unit()
        };
        let value = if let Some(value_ctx) = &var_def_ctx.value_ctx {
            Some(Box::new(self.low_expression_ctx(&value_ctx.value_ctx)?))
        } else {
            None
        };

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

    fn low_var_def_attributes(&self, ctx: &AttributesCtx) -> AstLowResult<VariableAttributes> {
        self.expect_incompatible_attribute(&ctx.safe_tkn)?;
        self.expect_incompatible_attribute(&ctx.unsafe_tkn)?;

        Ok(VariableAttributes {
            publicity: self.low_publicity_token(&ctx.pub_tkn),
        })
    }
}
