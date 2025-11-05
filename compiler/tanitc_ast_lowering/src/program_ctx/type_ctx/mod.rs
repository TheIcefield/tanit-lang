use tanitc_ast::program_ctx::type_ctx::TypeCtx;
use tanitc_hir::hir::types::TypeSpec;

use crate::{AstLowResult, AstLowering};

pub(crate) mod array_type_ctx;
pub(crate) mod func_type_ctx;
pub(crate) mod named_type_ctx;
pub(crate) mod never_type_ctx;
pub(crate) mod ptr_type_ctx;
pub(crate) mod ref_type_ctx;
pub(crate) mod tuple_type_ctx;

impl AstLowering {
    pub(crate) fn low_type_ctx(&self, type_ctx: &TypeCtx) -> AstLowResult<TypeSpec> {
        match type_ctx {
            TypeCtx::Array(ctx) => self.low_array_type_ctx(ctx),
            TypeCtx::Func(ctx) => self.low_func_type_ctx(ctx),
            TypeCtx::Named(ctx) => self.low_named_type_ctx(ctx),
            TypeCtx::Never(ctx) => self.low_never_type_ctx(ctx),
            TypeCtx::Ptr(ctx) => self.low_ptr_type_ctx(ctx),
            TypeCtx::Ref(ctx) => self.low_ref_type_ctx(ctx),
            TypeCtx::Tuple(ctx) => self.low_tuple_type_ctx(ctx),
        }
    }
}
