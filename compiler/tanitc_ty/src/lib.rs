use tanitc_ident::Ident;

use std::str::FromStr;

#[derive(Clone, PartialEq)]
pub enum Type {
    Ref(Box<Type>),
    Ptr(Box<Type>),
    Tuple(Vec<Type>),
    Array {
        size: Option<usize>,
        value_type: Box<Type>,
    },
    Template {
        identifier: Ident,
        generics: Vec<Type>,
    },
    Custom(String),
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

    pub fn get_c_type(&self) -> String {
        match self {
            Self::Auto => unreachable!("automatic type is not eliminated"),
            Self::Bool | Self::U8 => "unsigned char",
            Self::U16 => "unsigned short",
            Self::U32 => "unsigned int",
            Self::U64 => "unsigned long",
            Self::U128 => "unsigned long long",
            Self::I8 => "unsigned int",
            Self::I16 => "signed short",
            Self::I32 => "signed int",
            Self::I64 => "signed long",
            Self::I128 => "signed long long",
            Self::F32 => "float",
            Self::F64 => "double",
            Self::Custom(id) => id,
            Self::Tuple(components) => {
                if components.is_empty() {
                    "void"
                } else {
                    unimplemented!()
                }
            }
            _ => unimplemented!(),
        }
        .to_string()
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
            _ => Ok(Type::Custom(s.to_string())),
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
            Self::Ref(ref_to) => {
                write!(f, "&")?;

                write!(f, "{}", ref_to)
            }
            Self::Ptr(ptr_to) => {
                write!(f, "*")?;

                write!(f, "{}", ptr_to)
            }
            Self::Template {
                identifier,
                generics,
            } => {
                write!(f, "{}<", identifier)?;
                for generic in generics.iter() {
                    write!(f, "{}", generic)?;
                }
                write!(f, ">")
            }
            Self::Tuple(components) => {
                write!(f, "( ")?;

                for i in components.iter() {
                    write!(f, "{} ", i)?;
                }

                write!(f, ")")
            }
            Self::Array { value_type, .. } => write!(f, "[{}]", value_type),
            Self::Custom(s) => write!(f, "{}", s),

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
