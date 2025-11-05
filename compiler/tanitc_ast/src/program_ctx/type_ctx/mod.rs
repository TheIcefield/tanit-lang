use crate::program_ctx::type_ctx::{
    array_type_ctx::ArrayTypeCtx, func_type_ctx::FuncTypeCtx, named_type_ctx::NamedTypeCtx,
    never_type_ctx::NeverTypeCtx, ptr_type_ctx::PtrTypeCtx, ref_type_ctx::RefTypeCtx,
    tuple_type_ctx::TupleTypeCtx,
};

pub mod array_type_ctx;
pub mod func_type_ctx;
pub mod named_type_ctx;
pub mod never_type_ctx;
pub mod ptr_type_ctx;
pub mod ref_type_ctx;
pub mod tuple_type_ctx;

#[derive(Debug, Clone)]
pub enum TypeCtx {
    Named(NamedTypeCtx),
    Never(NeverTypeCtx),
    Ref(RefTypeCtx),
    Ptr(PtrTypeCtx),
    Func(FuncTypeCtx),
    Tuple(TupleTypeCtx),
    Array(ArrayTypeCtx),
}

impl TypeCtx {
    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::Named(_) => "named-type-ctx",
            Self::Never(_) => "never-type-ctx",
            Self::Ref(_) => "ref-type-ctx",
            Self::Ptr(_) => "ptr-type-ctx",
            Self::Func(_) => "func-type-ctx",
            Self::Tuple(_) => "tuple-type-ctx",
            Self::Array(_) => "arry-type-ctx",
        }
    }

    pub fn is_named(&self) -> bool {
        matches!(self, Self::Named(_))
    }

    pub fn is_never(&self) -> bool {
        matches!(self, Self::Never(_))
    }

    pub fn is_ref(&self) -> bool {
        matches!(self, Self::Ref(_))
    }

    pub fn is_ptr(&self) -> bool {
        matches!(self, Self::Ptr(_))
    }

    pub fn is_func(&self) -> bool {
        matches!(self, Self::Func(_))
    }

    pub fn is_tuple(&self) -> bool {
        matches!(self, Self::Tuple(_))
    }

    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array(_))
    }
}
