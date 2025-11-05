use tanitc_ast::program_ctx::statement_ctx::{
    attributes_ctx::AttributesCtx,
    definition_ctx::impl_def_ctx::{ImplDefBodyCtx, ImplDefCtx},
};
use tanitc_hir::hir::{
    definitions::{
        functions::FunctionDef,
        methods::{ImplAttributes, ImplDef},
        Definition,
    },
    Hir,
};
use tanitc_messages::Message;

use crate::{AstLowResult, AstLowering};

impl AstLowering {
    pub(crate) fn low_impl_def_ctx(&mut self, ctx: &ImplDefCtx) -> AstLowResult<ImplDef> {
        let location = ctx.impl_tkn.get_location();
        let attrs = self.low_impl_def_attributes(&ctx.attributes_ctx)?;
        let identifier = self.low_name_ctx(&ctx.name_ctx).id;
        let methods = self.low_impl_def_body_ctx(&ctx.body_ctx)?;

        Ok(ImplDef {
            location,
            attrs,
            identifier,
            methods,
        })
    }

    fn low_impl_def_attributes(&self, ctx: &AttributesCtx) -> AstLowResult<ImplAttributes> {
        self.expect_incompatible_attribute(&ctx.safe_tkn)?;
        self.expect_incompatible_attribute(&ctx.unsafe_tkn)?;
        self.expect_incompatible_attribute(&ctx.pub_tkn)?;

        Ok(ImplAttributes {})
    }

    fn low_impl_def_body_ctx(&mut self, ctx: &ImplDefBodyCtx) -> AstLowResult<Vec<FunctionDef>> {
        let body = self.low_block_ctx(&ctx.block_ctx)?;
        let mut methods = Vec::<FunctionDef>::with_capacity(body.statements.len());

        for statement in &body.statements {
            if !matches!(statement, Hir::Definition(Definition::Func(_))) {
                self.error(Message::from_string(
                    statement.location(),
                    format!("{} is not supported in impls", statement.kind_str()),
                ));
            }
        }

        methods.shrink_to_fit();
        Ok(methods)
    }
}
