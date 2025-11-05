use tanitc_ast::program_ctx::statement_ctx::{
    attributes_ctx::AttributesCtx,
    definition_ctx::enum_def_ctx::{
        EnumDefBodyCtx, EnumDefCtx, EnumDefUnitAssignCtx, EnumDefUnitCtx,
    },
};

use tanitc_hir::hir::definitions::enums::{EnumAttributes, EnumDef, EnumUnits};
use tanitc_ident::Ident;
use tanitc_lexer::token::lexeme::Lexeme;
use tanitc_messages::Message;

use crate::{AstLowResult, AstLowering};

impl AstLowering {
    pub(crate) fn low_enum_def_ctx(&mut self, enum_def_ctx: &EnumDefCtx) -> AstLowResult<EnumDef> {
        let location = enum_def_ctx.enum_tkn.get_location();
        let attributes = self.low_enum_def_attributes(&enum_def_ctx.attributes_ctx)?;
        let name = self.low_name_ctx(&enum_def_ctx.name_ctx);
        let fields = self.low_enum_def_body_ctx(&enum_def_ctx.body_ctx)?;

        Ok(EnumDef {
            location,
            attributes,
            name,
            fields,
        })
    }

    fn low_enum_def_attributes(&self, ctx: &AttributesCtx) -> AstLowResult<EnumAttributes> {
        self.expect_incompatible_attribute(&ctx.safe_tkn)?;
        self.expect_incompatible_attribute(&ctx.unsafe_tkn)?;

        Ok(EnumAttributes {
            publicity: self.low_publicity_token(&ctx.pub_tkn),
        })
    }

    fn low_enum_def_body_ctx(
        &mut self,
        enum_def_body_ctx: &EnumDefBodyCtx,
    ) -> AstLowResult<EnumUnits> {
        let mut units = EnumUnits::new();

        for (units_ctx, _) in enum_def_body_ctx.units_ctx.iter() {
            let Some(unit_ctx) = units_ctx else {
                continue;
            };

            let unit_res = self.low_enum_def_unit_ctx(unit_ctx);

            match unit_res {
                Ok((id, unit)) => {
                    units.insert(id, unit);
                }
                Err(err) => self.error(err),
            }
        }

        Ok(units)
    }

    fn low_enum_def_unit_ctx(
        &self,
        unit_ctx: &EnumDefUnitCtx,
    ) -> AstLowResult<(Ident, Option<usize>)> {
        let id = unit_ctx.name_ctx.identifier();
        let unit = if let Some(assign_ctx) = &unit_ctx.assign_ctx {
            Some(self.low_enum_def_unit_assign_ctx(assign_ctx)?)
        } else {
            None
        };

        Ok((id, unit))
    }

    fn low_enum_def_unit_assign_ctx(
        &self,
        assign_ctx: &EnumDefUnitAssignCtx,
    ) -> AstLowResult<usize> {
        let Lexeme::Integer(value) = assign_ctx.value_tkn.lexeme_ref() else {
            // Parser already returned unexpected_token
            return Ok(0);
        };

        match value.parse::<usize>() {
            Ok(val) => Ok(val),
            Err(err) => Err(Message::parse_int_error(
                assign_ctx.value_tkn.get_location(),
                err,
            )),
        }
    }
}
