use crate::program_ctx::statement_ctx::{
    attributes_ctx::AttributesCtx,
    definition_ctx::{
        alias_def_ctx::AliasDefCtx, const_def_ctx::ConstDefCtx, enum_def_ctx::EnumDefCtx,
        extern_ctx::ExternCtx, func_def_ctx::FuncDefCtx, impl_def_ctx::ImplDefCtx,
        module_def_ctx::ModuleDefCtx, static_def_ctx::StaticDefCtx, struct_def_ctx::StructDefCtx,
        union_def_ctx::UnionDefCtx, var_def_ctx::VarDefCtx, variant_def_ctx::VariantDefCtx,
    },
};

pub mod alias_def_ctx;
pub mod const_def_ctx;
pub mod enum_def_ctx;
pub mod extern_ctx;
pub mod func_def_ctx;
pub mod impl_def_ctx;
pub mod module_def_ctx;
pub mod static_def_ctx;
pub mod struct_def_ctx;
pub mod union_def_ctx;
pub mod var_def_ctx;
pub mod variant_def_ctx;

#[derive(Debug, Clone)]
pub enum DefinitionCtx {
    Alias(AliasDefCtx),
    Const(ConstDefCtx),
    Enum(EnumDefCtx),
    Func(FuncDefCtx),
    Module(ModuleDefCtx),
    Static(StaticDefCtx),
    Struct(StructDefCtx),
    Union(UnionDefCtx),
    Variable(VarDefCtx),
    Variant(VariantDefCtx),
    Impl(ImplDefCtx),
    Extern(ExternCtx),
}

impl DefinitionCtx {
    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::Alias(_) => "alias-def-ctx",
            Self::Const(_) => "const-def-ctx",
            Self::Enum(_) => "enum-def-ctx",
            Self::Func(_) => "func-def-ctx",
            Self::Module(_) => "module-def-ctx",
            Self::Static(_) => "static-def-ctx",
            Self::Struct(_) => "struct-def-ctx",
            Self::Union(_) => "union-def-ctx",
            Self::Variable(_) => "var-def-ctx",
            Self::Variant(_) => "variant-def-ctx",
            Self::Impl(_) => "impl-def-ctx",
            Self::Extern(_) => "extern-def-ctx",
        }
    }

    pub fn set_attributes(&mut self, attrs: AttributesCtx) {
        match self {
            Self::Alias(ctx) => *ctx.attributes_ctx = attrs,
            Self::Const(ctx) => *ctx.attributes_ctx = attrs,
            Self::Enum(ctx) => *ctx.attributes_ctx = attrs,
            Self::Func(ctx) => *ctx.attributes_ctx = attrs,
            Self::Module(ctx) => *ctx.attributes_ctx = attrs,
            Self::Static(ctx) => *ctx.attributes_ctx = attrs,
            Self::Struct(ctx) => *ctx.attributes_ctx = attrs,
            Self::Union(ctx) => *ctx.attributes_ctx = attrs,
            Self::Variable(ctx) => *ctx.attributes_ctx = attrs,
            Self::Variant(ctx) => *ctx.attributes_ctx = attrs,
            Self::Impl(ctx) => *ctx.attributes_ctx = attrs,
            Self::Extern(ctx) => *ctx.attributes_ctx = attrs,
        }
    }
}
