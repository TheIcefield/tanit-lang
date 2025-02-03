use crate::ast::{
    identifiers::{Identifier, IdentifierType},
    Ast,
};

use tanitc_messages::Message;

use std::str::FromStr;

pub mod analyzer;
pub mod codegen;
pub mod parser;
pub mod serialyzer;

#[derive(Clone, PartialEq)]
pub enum Type {
    Ref {
        is_mut: bool,
        ref_to: Box<Type>,
    },
    Ptr {
        is_mut: bool,
        ptr_to: Box<Type>,
    },
    Tuple {
        components: Vec<Type>,
    },
    Array {
        size: Option<Box<Ast>>,
        value_type: Box<Type>,
    },
    Template {
        identifier: Identifier,
        arguments: Vec<Type>,
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
}

impl Type {
    pub fn new() -> Self {
        Self::unit()
    }

    pub fn unit() -> Self {
        Self::Tuple {
            components: Vec::new(),
        }
    }

    pub fn from_id(id: &Identifier) -> Self {
        match &id.identifier {
            IdentifierType::Common(id) => Self::from_str(id).unwrap(),
            IdentifierType::Complex(..) => unimplemented!("creation type by complex id"),
        }
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
}

impl std::str::FromStr for Type {
    type Err = Message;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
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
            Self::Ref { is_mut, ref_to } => {
                write!(f, "&")?;

                if *is_mut {
                    write!(f, "mut")?;
                }

                write!(f, "{}", ref_to)
            }
            Self::Ptr { is_mut, ptr_to } => {
                write!(f, "*")?;

                if *is_mut {
                    write!(f, "mut")?;
                }

                write!(f, "{}", ptr_to)
            }
            Self::Template {
                identifier,
                arguments,
            } => {
                write!(f, "{}<", identifier)?;
                for i in arguments.iter() {
                    write!(f, "{}", i)?;
                }
                write!(f, ">")
            }
            Self::Tuple { components } => {
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
        }
    }
}

impl std::fmt::Debug for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self, f)
    }
}

impl From<Type> for Ast {
    fn from(value: Type) -> Self {
        Self::Type(value)
    }
}
