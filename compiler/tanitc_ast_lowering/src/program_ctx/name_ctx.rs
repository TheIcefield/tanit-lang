use tanitc_ast::program_ctx::name_ctx::NameCtx;
use tanitc_ident::Name;

use crate::AstLowering;

impl AstLowering {
    pub(crate) fn low_name_ctx(&self, name_ctx: &NameCtx) -> Name {
        Name::from(name_ctx.identifier())
    }
}
