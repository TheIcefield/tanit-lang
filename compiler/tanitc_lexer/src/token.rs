use super::location::Location;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Lexem {
    EndOfFile,
    EndOfLine,

    LParen,       // (
    RParen,       // )
    Lcb,          // {
    Rcb,          // }
    Lsb,          // [
    Rsb,          // ]
    Assign,       // =
    Plus,         // +
    AddAssign,    // +=
    Minus,        // -
    SubAssign,    // -=
    Star,         // *
    MulAssign,    // *=
    Slash,        // /
    DivAssign,    // /=
    Percent,      // %
    ModAssign,    // %=
    Eq,           // ==
    Neq,          // !=
    Not,          // !
    Lt,           // <
    Lte,          // <=
    Gt,           // >
    Gte,          // >=
    LShift,       // <<
    RShift,       // >>
    LShiftAssign, // <<=
    RShiftAssign, // >>=
    Stick,        // |
    Or,           // ||
    Ampersand,    // &
    And,          // &&
    Xor,          // ^
    OrAssign,     // |=
    AndAssign,    // &=
    XorAssign,    // ^=
    Comma,        // ,
    Dot,          // .
    Colon,        // :
    Dcolon,       // ::
    Arrow,        // ->

    KwLet,
    KwFunc,
    KwIf,
    KwElse,
    KwDo,
    KwWhile,
    KwFor,
    KwLoop,
    KwContinue,
    KwBreak,
    KwReturn,
    KwDef,
    KwModule,
    KwStruct,
    KwVariant,
    KwEnum,
    KwAlias,
    KwUse,
    KwExtern,
    KwStatic,
    KwMut,
    KwConst,
    KwAs,

    Identifier(String),
    Integer(String),
    Decimal(String),

    Unknown,
}

impl Lexem {
    pub fn new() -> Self {
        Self::Identifier(String::new())
    }

    pub fn from(v: &str) -> Self {
        Self::Identifier(String::from(v))
    }

    pub fn get_string(&self) -> String {
        match self {
            Self::Identifier(val) | Self::Integer(val) | Self::Decimal(val) => String::from(val),

            _ => String::new(),
        }
    }

    pub fn get_str(&self) -> Option<&str> {
        match self {
            Self::Identifier(val) | Self::Integer(val) | Self::Decimal(val) => Some(val),

            _ => None,
        }
    }

    pub fn get_int(&self) -> Option<usize> {
        match self {
            Self::Integer(val) => val.parse().ok(),
            _ => None,
        }
    }

    pub fn get_dec(&self) -> Option<f64> {
        match self {
            Self::Decimal(val) => val.parse().ok(),
            _ => None,
        }
    }
}

impl Default for Lexem {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for Lexem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EndOfFile => write!(f, "EOF"),
            Self::EndOfLine => write!(f, "EOL"),
            Self::LParen => write!(f, "("),
            Self::RParen => write!(f, ")"),
            Self::Lcb => write!(f, "{{"),
            Self::Rcb => write!(f, "}}"),
            Self::Lsb => write!(f, "["),
            Self::Rsb => write!(f, "]"),

            Self::Assign => write!(f, "="),
            Self::Plus => write!(f, "+"),
            Self::AddAssign => write!(f, "+="),
            Self::Minus => write!(f, "-"),
            Self::SubAssign => write!(f, "-="),
            Self::Star => write!(f, "*"),
            Self::MulAssign => write!(f, "*="),
            Self::Slash => write!(f, "/"),
            Self::DivAssign => write!(f, "/="),
            Self::Percent => write!(f, "%"),
            Self::ModAssign => write!(f, "%="),
            Self::Ampersand => write!(f, "&"),
            Self::And => write!(f, "&&"),
            Self::AndAssign => write!(f, "&="),
            Self::Stick => write!(f, "|"),
            Self::Or => write!(f, "||"),
            Self::OrAssign => write!(f, "|="),
            Self::Xor => write!(f, "^"),
            Self::XorAssign => write!(f, "^="),
            Self::Comma => write!(f, ","),
            Self::Dot => write!(f, "."),
            Self::Colon => write!(f, ":"),
            Self::Dcolon => write!(f, "::"),
            Self::Arrow => write!(f, "->"),

            Self::Lt => write!(f, "<"),
            Self::Lte => write!(f, "<="),
            Self::Gt => write!(f, ">"),
            Self::Gte => write!(f, ">="),
            Self::LShift => write!(f, "<<"),
            Self::LShiftAssign => write!(f, "<<="),
            Self::RShift => write!(f, ">>"),
            Self::RShiftAssign => write!(f, ">>="),
            Self::Eq => write!(f, "=="),
            Self::Neq => write!(f, "!="),
            Self::Not => write!(f, "!"),

            Self::KwLet => write!(f, "let"),
            Self::KwFunc => write!(f, "func"),
            Self::KwIf => write!(f, "if"),
            Self::KwElse => write!(f, "else"),
            Self::KwDo => write!(f, "do"),
            Self::KwWhile => write!(f, "while"),
            Self::KwFor => write!(f, "for"),
            Self::KwLoop => write!(f, "loop"),
            Self::KwContinue => write!(f, "continue"),
            Self::KwBreak => write!(f, "break"),
            Self::KwReturn => write!(f, "return"),
            Self::KwStruct => write!(f, "struct"),
            Self::KwVariant => write!(f, "variant"),
            Self::KwEnum => write!(f, "enum"),
            Self::KwAlias => write!(f, "alias"),
            Self::KwDef => write!(f, "def"),
            Self::KwModule => write!(f, "module"),
            Self::KwUse => write!(f, "use"),
            Self::KwExtern => write!(f, "extern"),
            Self::KwStatic => write!(f, "static"),
            Self::KwMut => write!(f, "mut"),
            Self::KwConst => write!(f, "const"),
            Self::KwAs => write!(f, "as"),

            Self::Identifier(val) => write!(f, "{}", val),
            Self::Integer(val) => write!(f, "{}", val),
            Self::Decimal(val) => write!(f, "{}", val),

            Self::Unknown => write!(f, "unknown token"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub lexem: Lexem,
    pub location: Location,
}

impl Token {
    pub fn new(lexem: Lexem, location: Location) -> Self {
        Self { lexem, location }
    }

    pub fn is_identifier(&self) -> bool {
        matches!(self.lexem, Lexem::Identifier(_))
    }

    pub fn is_integer(&self) -> bool {
        matches!(self.lexem, Lexem::Integer(_))
    }

    pub fn is_decimal(&self) -> bool {
        matches!(self.lexem, Lexem::Decimal(_))
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]: \"{}\"", self.location, self.lexem)
    }
}
