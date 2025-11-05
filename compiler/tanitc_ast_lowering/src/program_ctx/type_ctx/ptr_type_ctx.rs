use tanitc_ast::program_ctx::type_ctx::ptr_type_ctx::PtrTypeCtx;
use tanitc_attributes::Mutability;
use tanitc_hir::hir::types::{PtrType, Type, TypeSpec};
use tanitc_lexer::token::lexeme::Lexeme;

use crate::{AstLowResult, AstLowering};

impl AstLowering {
    pub(crate) fn low_ptr_type_ctx(&self, type_ctx: &PtrTypeCtx) -> AstLowResult<TypeSpec> {
        let location = type_ctx.star_tkn.get_location();

        let ptr_to = Box::new(self.low_type_ctx(&type_ctx.type_ctx)?.ty);
        let mutability = if Lexeme::KwMut == *type_ctx.mut_tkn.lexeme_ref() {
            Mutability::Mutable
        } else {
            Mutability::Immutable
        };

        let ty = Type::Ptr(PtrType { ptr_to, mutability });

        Ok(TypeSpec { location, ty })
    }
}
