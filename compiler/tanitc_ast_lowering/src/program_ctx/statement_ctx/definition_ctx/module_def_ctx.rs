use tanitc_ast::program_ctx::statement_ctx::{
    attributes_ctx::AttributesCtx,
    definition_ctx::module_def_ctx::{ModuleDefBodyCtx, ModuleDefCtx},
};
use tanitc_hir::hir::definitions::modules::{ModuleAttributes, ModuleDef, ModuleDefBody};

use crate::{AstLowResult, AstLowering};

impl AstLowering {
    pub(crate) fn low_module_def_ctx(
        &mut self,
        module_def_ctx: &ModuleDefCtx,
    ) -> AstLowResult<ModuleDef> {
        let location = module_def_ctx.module_tkn.get_location();
        let attributes = self.low_module_def_attributes(&module_def_ctx.attributes_ctx)?;
        let name = self.low_name_ctx(&module_def_ctx.name_ctx);

        let body = match &module_def_ctx.body_ctx {
            ModuleDefBodyCtx::Internal(ctx) => {
                ModuleDefBody::Internal(Box::new(self.low_block_ctx(ctx)?))
            }
            ModuleDefBodyCtx::External(ctx) => {
                ModuleDefBody::External(Box::new(self.low_program_ctx(ctx)?))
            }
        };

        Ok(ModuleDef {
            location,
            attributes,
            name,
            body,
        })
    }

    fn low_module_def_attributes(&self, ctx: &AttributesCtx) -> AstLowResult<ModuleAttributes> {
        Ok(ModuleAttributes {
            publicity: self.low_publicity_token(&ctx.pub_tkn),
            safety: self.low_safety(&ctx.safe_tkn, &ctx.unsafe_tkn)?,
        })
    }
}
