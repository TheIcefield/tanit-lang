use tanitc_ast::program_ctx::type_ctx::ref_type_ctx::RefTypeCtx;
use tanitc_attributes::Mutability;
use tanitc_hir::hir::type_spec::{RefType, Type, TypeSpec};
use tanitc_lexer::token::lexeme::Lexeme;

use crate::{AstLowResult, AstLowering};

impl AstLowering {
    pub(crate) fn low_ref_type_ctx(&self, type_ctx: &RefTypeCtx) -> AstLowResult<TypeSpec> {
        let location = type_ctx.ampersand_tkn.get_location();

        let ref_to = Box::new(self.low_type_ctx(&type_ctx.type_ctx)?.ty);
        let mutability = if type_ctx
            .mut_tkn
            .as_ref()
            .is_some_and(|tkn| Lexeme::KwMut == *tkn.lexeme_ref())
        {
            Mutability::Mutable
        } else {
            Mutability::Immutable
        };

        let ty = Type::Ref(RefType { ref_to, mutability });

        Ok(TypeSpec { location, ty })
    }
}
