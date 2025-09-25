use tanitc_attributes::Mutability;
use tanitc_ident::{Ident, Name};

use std::str::FromStr;

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
pub struct FuncType {
    pub parameters: Vec<Type>,
    pub return_type: Box<Type>,
}

#[derive(Clone, PartialEq)]
pub enum Type {
    Ref(RefType),
    Ptr(Box<Type>),
    Tuple(Vec<Type>),
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

impl Type {
    pub fn new() -> Self {
        Self::unit()
    }

    pub fn unit() -> Self {
        Self::Tuple(Vec::new())
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
        let Self::Tuple(components) = self else {
            return false;
        };

        components.is_empty()
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

    pub fn as_str(&self) -> String {
        match self {
            Self::Auto => "auto".to_string(),
            Self::Bool => "bool".to_string(),
            Self::U8 => "u8".to_string(),
            Self::U16 => "u16".to_string(),
            Self::U32 => "u32".to_string(),
            Self::U64 => "u64".to_string(),
            Self::U128 => "u128".to_string(),
            Self::I8 => "i8".to_string(),
            Self::I16 => "i16".to_string(),
            Self::I32 => "i32".to_string(),
            Self::I64 => "i64".to_string(),
            Self::I128 => "i128".to_string(),
            Self::F32 => "f32".to_string(),
            Self::F64 => "f64".to_string(),
            Self::Custom(id) => id.id.to_string(),
            Self::Ref(ref_type) => format!(
                "&{}{}",
                if ref_type.mutability.is_mutable() {
                    "mut "
                } else {
                    ""
                },
                ref_type.ref_to.as_str(),
            ),
            Self::Ptr(ptr_to) => format!("*{ptr_to}"),
            Self::Tuple(components) => {
                let mut res = String::new();

                res.push_str("( ");

                components.iter().for_each(|c_type| {
                    res.push_str(&format!("{}, ", c_type.as_str()));
                });

                res.push(')');

                res
            }
            Self::Array { value_type, .. } => value_type.get_c_type(),
            _ => unimplemented!(),
        }
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
            Self::Tuple(components) => {
                if components.is_empty() {
                    "void".to_string()
                } else {
                    let mut res = String::new();

                    res.push_str("struct { ");

                    components.iter().enumerate().for_each(|(c_idx, c_type)| {
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

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ref(ref_type) => {
                write!(
                    f,
                    "&{}",
                    if ref_type.mutability.is_mutable() {
                        "mut "
                    } else {
                        ""
                    }
                )?;

                write!(f, "{}", ref_type.ref_to)
            }
            Self::Ptr(ptr_to) => {
                write!(f, "*")?;

                write!(f, "{ptr_to}")
            }
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
            Self::Tuple(components) => {
                write!(f, "( ")?;

                for c in components.iter() {
                    write!(f, "{c} ")?;
                }

                write!(f, ")")
            }
            Self::Array { value_type, .. } => write!(f, "[{value_type}]"),
            Self::Func(func_type) => {
                write!(f, "func (")?;

                if let Some(first_param) = func_type.parameters.first() {
                    write!(f, "{first_param}")?;
                }
                for p in func_type.parameters.iter().skip(1) {
                    write!(f, ", {p}")?;
                }

                write!(f, ") -> {}", func_type.return_type)?;

                Ok(())
            }
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
