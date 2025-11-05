use tanitc_ast::program_ctx::statement_ctx::use_ctx::UseCtx;
use tanitc_hir::hir::uses::{Use, UseIdentifier};
use tanitc_ident::Ident;

use crate::{AstLowResult, AstLowering};

impl AstLowering {
    pub(crate) fn low_use_ctx(&self, ctx: &UseCtx) -> AstLowResult<Use> {
        let location = ctx.use_tkn.get_location();
        let identifiers = self.low_use_idents(&ctx.idents)?;

        Ok(Use {
            location,
            identifiers,
        })
    }

    fn low_use_idents(&self, identifiers: &[Ident]) -> AstLowResult<Vec<UseIdentifier>> {
        let mut ids = Vec::<UseIdentifier>::new();

        for id in identifiers {
            let id_str = id.to_string();
            let id_comp = match &id_str[..] {
                "*" => UseIdentifier::UseAll,
                "crate" => UseIdentifier::UseCrate,
                "self" => UseIdentifier::UseSelf,
                "super" => UseIdentifier::UseSuper,
                _ => UseIdentifier::UseId(*id),
            };

            ids.push(id_comp);
        }

        Ok(ids)
    }
}
