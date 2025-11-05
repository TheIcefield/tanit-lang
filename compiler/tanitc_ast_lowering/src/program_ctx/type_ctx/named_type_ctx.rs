use tanitc_ast::program_ctx::type_ctx::named_type_ctx::NamedTypeCtx;
use tanitc_hir::hir::types::{Type, TypeSpec};
use tanitc_ident::Name;

use crate::{AstLowResult, AstLowering};

impl AstLowering {
    pub(crate) fn low_named_type_ctx(&self, type_ctx: &NamedTypeCtx) -> AstLowResult<TypeSpec> {
        let ty_id = type_ctx.name_ctx.identifier();
        let ty = match &ty_id.to_string()[..] {
            "i8" => Type::I8,
            "i16" => Type::I16,
            "i32" => Type::I32,
            "i64" => Type::I64,
            "i128" => Type::I128,
            "u8" => Type::U8,
            "u16" => Type::U16,
            "u32" => Type::U32,
            "u64" => Type::U64,
            "u128" => Type::U128,
            "f32" => Type::F32,
            "f64" => Type::F64,
            _ => Type::Custom(Name::from(ty_id)),
        };

        Ok(TypeSpec {
            location: type_ctx.name_ctx.name_tkn.get_location(),
            ty,
        })
    }
}
