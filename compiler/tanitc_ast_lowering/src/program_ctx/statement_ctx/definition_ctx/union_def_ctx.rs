use tanitc_ast::program_ctx::statement_ctx::{
    attributes_ctx::AttributesCtx,
    definition_ctx::union_def_ctx::{UnionDefBodyCtx, UnionDefCtx, UnionDefFieldCtx},
};

use tanitc_hir::hir::definitions::unions::{
    UnionAttributes, UnionDef, UnionFieldAttributes, UnionFieldInfo, UnionFieldsInfo,
};
use tanitc_ident::Ident;

use crate::{AstLowResult, AstLowering};

impl AstLowering {
    pub(crate) fn low_union_def_ctx(
        &mut self,
        union_def_ctx: &UnionDefCtx,
    ) -> AstLowResult<UnionDef> {
        Ok(UnionDef {
            location: union_def_ctx.union_tkn.get_location(),
            attributes: self.low_union_def_attributes(&union_def_ctx.attributes_ctx)?,
            name: self.low_name_ctx(&union_def_ctx.name_ctx),
            fields: self.low_union_def_body_ctx(&union_def_ctx.body_ctx)?,
            internals: Vec::new(),
        })
    }

    fn low_union_def_body_ctx(
        &mut self,
        union_def_body_ctx: &UnionDefBodyCtx,
    ) -> AstLowResult<UnionFieldsInfo> {
        let mut fields = UnionFieldsInfo::new();

        for (field_ctx, _) in union_def_body_ctx.fields_ctx.iter() {
            let Some(field_ctx) = field_ctx else {
                continue;
            };

            let (id, field) = self.low_union_def_field_ctx(field_ctx)?;

            fields.insert(id, field);
        }

        Ok(fields)
    }

    fn low_union_def_field_ctx(
        &mut self,
        field_ctx: &UnionDefFieldCtx,
    ) -> AstLowResult<(Ident, UnionFieldInfo)> {
        let id = field_ctx.name_ctx.identifier();
        let ty = self.low_type_ctx(&field_ctx.type_ctx)?;

        Ok((
            id,
            UnionFieldInfo {
                attributes: UnionFieldAttributes::default(),
                ty,
            },
        ))
    }

    fn low_union_def_attributes(&self, ctx: &AttributesCtx) -> AstLowResult<UnionAttributes> {
        self.expect_incompatible_attribute(&ctx.safe_tkn)?;
        self.expect_incompatible_attribute(&ctx.unsafe_tkn)?;

        Ok(UnionAttributes {
            publicity: self.low_publicity_token(&ctx.pub_tkn),
        })
    }
}
