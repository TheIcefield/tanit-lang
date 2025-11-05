use tanitc_ast::program_ctx::statement_ctx::definition_ctx::DefinitionCtx;
use tanitc_hir::hir::definitions::Definition;

use crate::{AstLowResult, AstLowering};

pub(crate) mod alias_def_ctx;
pub(crate) mod const_def_ctx;
pub(crate) mod enum_def_ctx;
pub(crate) mod extern_ctx;
pub(crate) mod func_def_ctx;
pub(crate) mod impl_def_ctx;
pub(crate) mod module_def_ctx;
pub(crate) mod static_def_ctx;
pub(crate) mod struct_def_ctx;
pub(crate) mod union_def_ctx;
pub(crate) mod variable_def_ctx;
pub(crate) mod variant_def_ctx;

impl AstLowering {
    pub(crate) fn low_definition_ctx(
        &mut self,
        definition_ctx: &DefinitionCtx,
    ) -> AstLowResult<Definition> {
        match definition_ctx {
            DefinitionCtx::Alias(ctx) => self.low_alias_def_ctx(ctx).map(Definition::Alias),
            DefinitionCtx::Const(ctx) => self.low_const_def_ctx(ctx).map(Definition::Variable),
            DefinitionCtx::Enum(ctx) => self.low_enum_def_ctx(ctx).map(Definition::Enum),
            DefinitionCtx::Func(ctx) => self.low_func_def_ctx(ctx).map(Definition::Func),
            DefinitionCtx::Impl(ctx) => self.low_impl_def_ctx(ctx).map(Definition::Impl),
            DefinitionCtx::Module(ctx) => self.low_module_def_ctx(ctx).map(Definition::Module),
            DefinitionCtx::Static(ctx) => self.low_static_def_ctx(ctx).map(Definition::Variable),
            DefinitionCtx::Struct(ctx) => self.low_struct_def_ctx(ctx).map(Definition::Struct),
            DefinitionCtx::Union(ctx) => self.low_union_def_ctx(ctx).map(Definition::Union),
            DefinitionCtx::Variable(ctx) => {
                self.low_variable_def_ctx(ctx).map(Definition::Variable)
            }
            DefinitionCtx::Variant(ctx) => self.low_variant_def_ctx(ctx).map(Definition::Variant),
            DefinitionCtx::Extern(ctx) => self.low_extern_def_ctx(ctx).map(Definition::Extern),
        }
    }
}
