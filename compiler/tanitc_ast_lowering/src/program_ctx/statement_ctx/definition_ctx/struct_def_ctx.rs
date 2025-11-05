use tanitc_ast::program_ctx::statement_ctx::{
    attributes_ctx::AttributesCtx,
    definition_ctx::struct_def_ctx::{StructDefBodyCtx, StructDefCtx, StructDefFieldCtx},
};

use tanitc_hir::hir::definitions::structs::{
    StructAttributes, StructDef, StructFieldAttributes, StructFieldInfo, StructFieldsInfo,
};
use tanitc_ident::Ident;

use crate::{AstLowResult, AstLowering};

impl AstLowering {
    pub(crate) fn low_struct_def_ctx(
        &mut self,
        struct_def_ctx: &StructDefCtx,
    ) -> AstLowResult<StructDef> {
        Ok(StructDef {
            location: struct_def_ctx.struct_tkn.get_location(),
            attributes: self.low_struct_def_attributes(&struct_def_ctx.attributes_ctx)?,
            name: self.low_name_ctx(&struct_def_ctx.name_ctx),
            fields: self.low_struct_def_body_ctx(&struct_def_ctx.body_ctx)?,
            internals: Vec::new(),
        })
    }

    pub(crate) fn low_struct_def_body_ctx(
        &mut self,
        struct_def_body_ctx: &StructDefBodyCtx,
    ) -> AstLowResult<StructFieldsInfo> {
        let mut fields = StructFieldsInfo::new();

        for (field_ctx, _) in struct_def_body_ctx.fields_ctx.iter() {
            let Some(field_ctx) = field_ctx else {
                continue;
            };

            let (id, field) = self.low_struct_def_field_ctx(field_ctx)?;

            fields.insert(id, field);
        }

        Ok(fields)
    }

    fn low_struct_def_field_ctx(
        &mut self,
        field_ctx: &StructDefFieldCtx,
    ) -> AstLowResult<(Ident, StructFieldInfo)> {
        let id = field_ctx.name_ctx.identifier();
        let ty = self.low_type_ctx(&field_ctx.type_ctx)?;

        Ok((
            id,
            StructFieldInfo {
                attributes: StructFieldAttributes::default(),
                ty,
            },
        ))
    }

    fn low_struct_def_attributes(&self, ctx: &AttributesCtx) -> AstLowResult<StructAttributes> {
        self.expect_incompatible_attribute(&ctx.safe_tkn)?;
        self.expect_incompatible_attribute(&ctx.unsafe_tkn)?;

        Ok(StructAttributes {
            publicity: self.low_publicity_token(&ctx.pub_tkn),
        })
    }
}
