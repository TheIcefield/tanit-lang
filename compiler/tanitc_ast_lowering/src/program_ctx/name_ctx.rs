use tanitc_ast::program_ctx::name_ctx::{NameCtx, NameSpecCtx};
use tanitc_messages::Message;
use tanitc_name::NameSpec;

use crate::{AstLowResult, AstLowering};

impl AstLowering {
    pub(crate) fn low_name_ctx(&self, name_ctx: &NameCtx) -> NameSpec {
        NameSpec {
            location: name_ctx.name_tkn.get_location(),
            path: vec![name_ctx.identifier().into()],
        }
    }

    pub(crate) fn low_name_spec_ctx(&self, name_spec_ctx: &NameSpecCtx) -> AstLowResult<NameSpec> {
        let (first, _) = name_spec_ctx.names.first().ok_or(Message {
            location: None,
            text: "Empty name-spec ctx".to_string(),
        })?;

        let location = first.get_location();
        let path = name_spec_ctx
            .names
            .iter()
            .map(|(id_tkn, _)| id_tkn.identifier().into())
            .collect();

        Ok(NameSpec { location, path })
    }
}
