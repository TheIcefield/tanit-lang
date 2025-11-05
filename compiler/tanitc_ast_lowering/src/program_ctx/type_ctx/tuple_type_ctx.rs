use tanitc_ast::program_ctx::type_ctx::tuple_type_ctx::{TupleTypeCtx, TupleTypeUnitCtx};
use tanitc_hir::hir::types::{TupleType, Type, TypeSpec};

use crate::{AstLowResult, AstLowering};

impl AstLowering {
    pub(crate) fn low_tuple_type_ctx(&self, type_ctx: &TupleTypeCtx) -> AstLowResult<TypeSpec> {
        let location = type_ctx.lparen_tkn.get_location();

        let units = self.low_tuple_type_units(&type_ctx.units_ctx)?;

        let ty = Type::Tuple(TupleType { units });

        Ok(TypeSpec { location, ty })
    }

    fn low_tuple_type_units(&self, units_ctx: &[TupleTypeUnitCtx]) -> AstLowResult<Vec<Type>> {
        let mut units = Vec::<Type>::new();

        for unit in units_ctx.iter() {
            units.push(self.low_type_ctx(&unit.type_ctx)?.ty);
        }

        Ok(units)
    }
}
