use tanitc_ast::program_ctx::statement_ctx::{
    attributes_ctx::AttributesCtx,
    definition_ctx::variant_def_ctx::{
        VariantDefBodyCtx, VariantDefCtx, VariantDefEnumFieldCtx, VariantDefFieldCtx,
        VariantDefStructFieldCtx, VariantDefTupleFieldCtx,
    },
};
use tanitc_hir::hir::{
    definitions::variants::{VariantAttributes, VariantDef, VariantField, VariantFields},
    type_spec::Type,
};
use tanitc_ident::Ident;
use tanitc_messages::Message;

use crate::{AstLowResult, AstLowering};

impl AstLowering {
    pub(crate) fn low_variant_def_ctx(
        &mut self,
        variant_def_ctx: &VariantDefCtx,
    ) -> AstLowResult<VariantDef> {
        let location = variant_def_ctx.variant_tkn.get_location();
        let attributes = self.low_variant_def_attributes(&variant_def_ctx.attributes_ctx)?;
        let name = self.low_name_ctx(&variant_def_ctx.name_ctx);
        let fields = self.low_variant_def_body_ctx(&variant_def_ctx.body_ctx)?;

        Ok(VariantDef {
            location,
            attributes,
            name,
            fields,
            internals: Vec::new(),
        })
    }

    fn low_variant_def_attributes(&self, ctx: &AttributesCtx) -> AstLowResult<VariantAttributes> {
        self.expect_incompatible_attribute(&ctx.safe_tkn)?;
        self.expect_incompatible_attribute(&ctx.unsafe_tkn)?;

        Ok(VariantAttributes {
            publicity: self.low_publicity_token(&ctx.pub_tkn),
        })
    }

    fn low_variant_def_body_ctx(
        &mut self,
        variant_def_body_ctx: &VariantDefBodyCtx,
    ) -> AstLowResult<VariantFields> {
        let mut fields = VariantFields::new();

        for (field_ctx, _) in variant_def_body_ctx.fields_ctx.iter() {
            let Some(field_ctx) = field_ctx else {
                continue;
            };

            match self.low_variant_def_field_ctx(field_ctx) {
                Ok((id, field)) => {
                    fields.insert(id, field);
                }
                Err(err) => self.error(err),
            }
        }

        Ok(fields)
    }

    fn low_variant_def_field_ctx(
        &mut self,
        field_ctx: &VariantDefFieldCtx,
    ) -> AstLowResult<(Ident, VariantField)> {
        match field_ctx {
            VariantDefFieldCtx::Enum(ctx) => self.low_variant_def_enum_field_ctx(ctx),
            VariantDefFieldCtx::Struct(ctx) => self.low_variant_def_struct_field_ctx(ctx),
            VariantDefFieldCtx::Tuple(ctx) => self.low_variant_def_tuple_field_ctx(ctx),
        }
    }

    fn low_variant_def_enum_field_ctx(
        &mut self,
        field_ctx: &VariantDefEnumFieldCtx,
    ) -> AstLowResult<(Ident, VariantField)> {
        let id = field_ctx.name_ctx.identifier();

        Ok((id, VariantField::Enum))
    }

    fn low_variant_def_struct_field_ctx(
        &mut self,
        field_ctx: &VariantDefStructFieldCtx,
    ) -> AstLowResult<(Ident, VariantField)> {
        let id = field_ctx.name_ctx.identifier();
        let fields = self.low_struct_def_body_ctx(&field_ctx.struct_body_ctx)?;

        Ok((id, VariantField::Struct(fields)))
    }

    fn low_variant_def_tuple_field_ctx(
        &mut self,
        field_ctx: &VariantDefTupleFieldCtx,
    ) -> AstLowResult<(Ident, VariantField)> {
        let id = field_ctx.name_ctx.identifier();
        let ty = self.low_tuple_type_ctx(&field_ctx.tuple_type_ctx)?;

        let Type::Tuple(units) = ty.ty else {
            return Err(Message::unreachable(
                field_ctx.tuple_type_ctx.lparen_tkn.get_location(),
                format!("Unexpected variant tuple field type: {ty:?}"),
            ));
        };

        Ok((id, VariantField::Tuple(units.units)))
    }
}
