use crate::hir::expressions::Expression;
use tanitc_ident::Ident;
use tanitc_lexer::location::Location;
use tanitc_name::NameSpec;

#[derive(Debug, Clone, PartialEq)]
pub struct Integer {
    pub location: Location,
    pub value: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Decimal {
    pub location: Location,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Text {
    pub location: Location,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayLiteral {
    pub location: Location,
    pub elements: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TupleLiteral {
    pub location: Location,
    pub units: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructLiteral {
    pub location: Location,
    pub name: NameSpec,
    pub fields: Vec<(Ident, Expression)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Integer(Integer),
    Decimal(Decimal),
    Text(Text),
    Array(ArrayLiteral),
    Tuple(TupleLiteral),
    Struct(StructLiteral),
}

impl Literal {
    pub fn location(&self) -> Location {
        match self {
            Self::Integer(lit) => lit.location,
            Self::Decimal(lit) => lit.location,
            Self::Text(lit) => lit.location,
            Self::Array(lit) => lit.location,
            Self::Tuple(lit) => lit.location,
            Self::Struct(lit) => lit.location,
        }
    }

    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::Integer(_) => "integer-literal",
            Self::Decimal(_) => "decimal-literal",
            Self::Text(_) => "text-literal",
            Self::Array(_) => "array-literal",
            Self::Tuple(_) => "tuple-literal",
            Self::Struct(_) => "struct-literal",
        }
    }
}
