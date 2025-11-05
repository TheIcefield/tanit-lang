use std::str::FromStr;
use tanitc_attributes::{Mutability, Safety};
use tanitc_ident::{Ident, Name};
use tanitc_lexer::location::Location;

use crate::hir::Hir;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct TypeSpec {
    pub location: Location,
    pub ty: Type,
}

impl TypeSpec {
    pub fn get_type(&self) -> Type {
        self.ty.clone()
    }

    pub fn get_c_type(&self) -> String {
        self.ty.get_c_type()
    }
}

impl From<TypeSpec> for Hir {
    fn from(value: TypeSpec) -> Self {
        Self::TypeSpec(value)
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum ArraySize {
    #[default]
    Unknown,
    Fixed(usize),
}

#[derive(Clone, PartialEq)]
pub struct RefType {
    pub ref_to: Box<Type>,
    pub mutability: Mutability,
}

#[derive(Clone, PartialEq)]
pub struct PtrType {
    pub ptr_to: Box<Type>,
    pub mutability: Mutability,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FuncTypeParam {
    pub ty: Box<Type>,
    pub id: Option<Ident>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FuncType {
    pub parameters: Vec<FuncTypeParam>,
    pub return_type: Box<Type>,
    pub safety: Safety,
}

#[derive(Default, Clone, PartialEq)]
pub struct TupleType {
    pub units: Vec<Type>,
}

#[derive(Clone, PartialEq)]
pub enum Type {
    Ref(RefType),
    Ptr(PtrType),
    Tuple(TupleType),
    Array {
        size: ArraySize,
        value_type: Box<Type>,
    },
    Template {
        identifier: Ident,
        generics: Vec<Type>,
    },
    Custom(Name),
    Func(FuncType),
    Auto,
    Bool,
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
    F32,
    F64,
    Str,
    Never,
}

impl From<Ident> for Type {
    fn from(value: Ident) -> Self {
        let s: String = value.into();
        Self::from_str(&s).unwrap()
    }
}

impl From<Name> for Type {
    fn from(value: Name) -> Self {
        Type::Custom(value)
    }
}

impl Type {
    pub fn new() -> Self {
        Self::unit()
    }

    pub fn unit() -> Self {
        Self::Tuple(TupleType::default())
    }

    pub fn is_common(&self) -> bool {
        matches!(
            self,
            Self::Bool
                | Self::F32
                | Self::F64
                | Self::I8
                | Self::I16
                | Self::I32
                | Self::I64
                | Self::I128
                | Self::U8
                | Self::U16
                | Self::U32
                | Self::U64
                | Self::U128
        )
    }

    pub fn is_unit(&self) -> bool {
        let Self::Tuple(typle_type) = self else {
            return false;
        };

        typle_type.units.is_empty()
    }

    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            Self::I8
                | Self::I16
                | Self::I32
                | Self::I64
                | Self::I128
                | Self::U8
                | Self::U16
                | Self::U32
                | Self::U64
                | Self::U128
        )
    }

    pub fn is_reference(&self) -> bool {
        matches!(self, Self::Ref { .. })
    }

    pub fn is_pointer(&self) -> bool {
        matches!(self, Self::Ptr(_))
    }

    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array { .. })
    }

    pub fn get_c_type(&self) -> String {
        match self {
            Self::Auto => unreachable!("automatic type is not eliminated"),
            Self::Bool | Self::U8 => "unsigned char".to_string(),
            Self::U16 => "unsigned short".to_string(),
            Self::U32 => "unsigned int".to_string(),
            Self::U64 => "unsigned long".to_string(),
            Self::U128 => "unsigned long long".to_string(),
            Self::I8 => "unsigned int".to_string(),
            Self::I16 => "signed short".to_string(),
            Self::I32 => "signed int".to_string(),
            Self::I64 => "signed long".to_string(),
            Self::I128 => "signed long long".to_string(),
            Self::F32 => "float".to_string(),
            Self::F64 => "double".to_string(),
            Self::Str => "char".to_string(),
            Self::Custom(id) => id.to_string(),
            Self::Ref(ref_type) => format!(
                "{}{}*",
                ref_type.ref_to.get_c_type(),
                if ref_type.mutability.is_const() {
                    " const "
                } else {
                    " "
                }
            ),
            Self::Tuple(tuple_type) => {
                if tuple_type.units.is_empty() {
                    "void".to_string()
                } else {
                    let mut res = String::new();

                    res.push_str("struct { ");

                    tuple_type
                        .units
                        .iter()
                        .enumerate()
                        .for_each(|(c_idx, c_type)| {
                            res.push_str(&format!("{} _{c_idx}; ", c_type.get_c_type()));
                        });

                    res.push('}');

                    res
                }
            }
            Self::Array { value_type, .. } => value_type.get_c_type(),
            _ => unimplemented!(),
        }
    }
}

impl std::str::FromStr for Type {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "!" => Ok(Type::Never),
            "bool" => Ok(Type::Bool),
            "i8" => Ok(Type::I8),
            "i16" => Ok(Type::I16),
            "i32" => Ok(Type::I32),
            "i64" => Ok(Type::I64),
            "i128" => Ok(Type::I128),
            "u8" => Ok(Type::U8),
            "u16" => Ok(Type::U16),
            "u32" => Ok(Type::U32),
            "u64" => Ok(Type::U64),
            "u128" => Ok(Type::U128),
            "f32" => Ok(Type::F32),
            "f64" => Ok(Type::F64),
            "str" => Ok(Type::Str),
            _ => Ok(Type::Custom(Name::from(s.to_string()))),
        }
    }
}

impl Default for Type {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for RefType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "&{}",
            if self.mutability.is_mutable() {
                "mut "
            } else {
                ""
            }
        )?;

        write!(f, "{}", self.ref_to)
    }
}

impl std::fmt::Display for PtrType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "*{}",
            if self.mutability.is_mutable() {
                "mut "
            } else {
                "const "
            }
        )?;

        write!(f, "{}", self.ptr_to)
    }
}

impl std::fmt::Display for TupleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "( ")?;

        for c in self.units.iter() {
            write!(f, "{c} ")?;
        }

        write!(f, ")")
    }
}

impl std::fmt::Display for FuncTypeParam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(id) = self.id {
            write!(f, "{id}:")?;
        }

        write!(f, "{}", self.ty)
    }
}

impl std::fmt::Display for FuncType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "func (")?;

        if let Some(first_param) = self.parameters.first() {
            write!(f, "{first_param}")?;
        }
        for p in self.parameters.iter().skip(1) {
            write!(f, ", {p}")?;
        }

        write!(f, ") -> {}", self.return_type)
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ref(ref_type) => write!(f, "{ref_type}"),
            Self::Ptr(ptr_type) => write!(f, "{ptr_type}"),
            Self::Tuple(tuple_type) => write!(f, "{tuple_type}"),
            Self::Template {
                identifier,
                generics,
            } => {
                write!(f, "{identifier}<")?;
                for generic in generics.iter() {
                    write!(f, "{generic}")?;
                }
                write!(f, ">")
            }
            Self::Array { value_type, .. } => write!(f, "[{value_type}]"),
            Self::Func(func_type) => write!(f, "{func_type}"),
            Self::Custom(s) => write!(f, "{s}"),
            Self::Auto => write!(f, "@auto"),
            Self::Bool => write!(f, "bool"),
            Self::I8 => write!(f, "i8"),
            Self::I16 => write!(f, "i16"),
            Self::I32 => write!(f, "i32"),
            Self::I64 => write!(f, "i64"),
            Self::I128 => write!(f, "i128"),
            Self::U8 => write!(f, "u8"),
            Self::U16 => write!(f, "u16"),
            Self::U32 => write!(f, "u32"),
            Self::U64 => write!(f, "u64"),
            Self::U128 => write!(f, "u128"),
            Self::F32 => write!(f, "f32"),
            Self::F64 => write!(f, "f64"),
            Self::Str => write!(f, "str"),
            Self::Never => write!(f, "!"),
        }
    }
}

impl std::fmt::Debug for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self, f)
    }
}
