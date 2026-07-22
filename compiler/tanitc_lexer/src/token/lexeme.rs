//! Lexeme (token kind) definitions.

use std::fmt::Display;

use tanitc_ident::Ident;

/// The kind of a lexer token.
///
/// `Lexeme` classifies every token produced by the [`Lexer`](crate::Lexer)
/// into one of the following categories:
///
/// - **Structural** — delimiters and punctuation (`LParen`, `RParen`, `Comma`, etc.)
/// - **Operators** — arithmetic, comparison, bitwise, and assignment operators
/// - **Keywords** — reserved words of the Tanit language (`KwVar`, `KwFunc`, etc.)
/// - **Literals** — identifiers, integer/decimal numbers, and string text
/// - **Special** — `EndOfLine` (newline) and `Unknown` (unrecognized character)
///
/// # Example
///
/// ```
/// use tanitc_lexer::token::lexeme::Lexeme;
///
/// let lex = Lexeme::identifier("foo");
/// assert!(lex.is_identifier());
/// assert_eq!(lex.to_string(), "foo");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Lexeme {
    /// End-of-line marker (newline character).
    EndOfLine,

    // ── Delimiters ──────────────────────────────────────────────
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `{`
    Lcb,
    /// `}`
    Rcb,
    /// `[`
    Lsb,
    /// `]`
    Rsb,

    // ── Operators ───────────────────────────────────────────────
    /// `=`
    Assign,
    /// `+`
    Plus,
    /// `+=`
    AddAssign,
    /// `-`
    Minus,
    /// `-=`
    SubAssign,
    /// `*`
    Star,
    /// `*=`
    MulAssign,
    /// `/`
    Slash,
    /// `/=`
    DivAssign,
    /// `%`
    Percent,
    /// `%=`
    ModAssign,
    /// `==`
    Eq,
    /// `!=`
    Neq,
    /// `!`
    Not,
    /// `<`
    Lt,
    /// `<=`
    Lte,
    /// `>`
    Gt,
    /// `>=`
    Gte,
    /// `<<`
    LShift,
    /// `>>`
    RShift,
    /// `<<=`
    LShiftAssign,
    /// `>>=`
    RShiftAssign,
    /// `|`
    Stick,
    /// `||`
    Or,
    /// `&`
    Ampersand,
    /// `&&`
    And,
    /// `^`
    Xor,
    /// `|=`
    OrAssign,
    /// `&=`
    AndAssign,
    /// `^=`
    XorAssign,
    /// `,`
    Comma,
    /// `.`
    Dot,
    /// `:`
    Colon,
    /// `::`
    Dcolon,

    // ── Keywords ────────────────────────────────────────────────
    /// `def`
    KwDef,
    /// `module`
    KwModule,
    /// `struct`
    KwStruct,
    /// `union`
    KwUnion,
    /// `variant`
    KwVariant,
    /// `impl`
    KwImpl,
    /// `enum`
    KwEnum,
    /// `var`
    KwVar,
    /// `mut`
    KwMut,
    /// `const`
    KwConst,
    /// `alias`
    KwAlias,
    /// `func`
    KwFunc,
    /// `if`
    KwIf,
    /// `else`
    KwElse,
    /// `loop`
    KwLoop,
    /// `do`
    KwDo,
    /// `while`
    KwWhile,
    /// `for`
    KwFor,
    /// `continue`
    KwContinue,
    /// `break`
    KwBreak,
    /// `return`
    KwReturn,
    /// `extern`
    KwExtern,
    /// `static`
    KwStatic,
    /// `use`
    KwUse,
    /// `super`
    KwSuper,
    /// `self`
    KwSelf,
    /// `crate`
    KwCrate,
    /// `as`
    KwAs,
    /// `safe`
    KwSafe,
    /// `unsafe`
    KwUnsafe,
    /// `pub`
    KwPub,

    // ── Literals ────────────────────────────────────────────────
    /// An integer literal (e.g. `42`), stored as its source text.
    Integer(String),
    /// A decimal (floating-point) literal (e.g. `3.14`), stored as its source text.
    Decimal(String),
    /// A string literal (e.g. `"hello"`), stored with surrounding quotes.
    Text(String),

    /// An identifier (user-defined name), backed by an interned [`Ident`].
    Identifier(Ident),

    /// An unrecognized character.
    Unknown,
}

impl Lexeme {
    /// Creates an `Unknown` lexeme.
    pub fn new() -> Self {
        Self::Unknown
    }

    /// Creates an [`Identifier`](Self::Identifier) lexeme from a string slice.
    pub fn identifier(v: &str) -> Self {
        Self::Identifier(Ident::from(v.to_string()))
    }

    /// Creates an [`Integer`](Self::Integer) lexeme from a `usize` value.
    pub fn integer(v: usize) -> Self {
        Self::Integer(v.to_string())
    }

    /// Creates a [`Decimal`](Self::Decimal) lexeme from an `f64` value.
    pub fn decimal(v: f64) -> Self {
        Self::Decimal(v.to_string())
    }

    /// Returns `true` if this lexeme is an identifier.
    pub fn is_identifier(&self) -> bool {
        matches!(self, Self::Identifier(_))
    }

    /// Returns `true` if this lexeme is an integer literal.
    pub fn is_integer(&self) -> bool {
        matches!(self, Self::Integer(_))
    }

    /// Returns `true` if this lexeme is a decimal literal.
    pub fn is_decimal(&self) -> bool {
        matches!(self, Self::Decimal(_))
    }

    /// Returns the string content of an identifier, integer, or decimal lexeme.
    ///
    /// Returns an empty string for all other variants.
    pub fn get_string(&self) -> String {
        match self {
            Self::Identifier(val) => val.to_string(),
            Self::Integer(val) | Self::Decimal(val) => val.clone(),

            _ => String::new(),
        }
    }

    /// Returns the raw string slice of an integer or decimal literal.
    ///
    /// Returns `None` for all other variants.
    pub fn get_str(&self) -> Option<&str> {
        match self {
            Self::Integer(val) | Self::Decimal(val) => Some(val),

            _ => None,
        }
    }

    /// Parses and returns the value of an integer literal.
    ///
    /// Returns `None` if the lexeme is not an integer or parsing fails.
    pub fn get_int(&self) -> Option<usize> {
        match self {
            Self::Integer(val) => val.parse().ok(),
            _ => None,
        }
    }

    /// Parses and returns the value of a decimal literal.
    ///
    /// Returns `None` if the lexeme is not a decimal or parsing fails.
    pub fn get_dec(&self) -> Option<f64> {
        match self {
            Self::Decimal(val) => val.parse().ok(),
            _ => None,
        }
    }
}

impl Default for Lexeme {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for Lexeme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EndOfLine => write!(f, "EOL"),
            Self::LParen => write!(f, "'('"),
            Self::RParen => write!(f, "')'"),
            Self::Lcb => write!(f, "'{{'"),
            Self::Rcb => write!(f, "'}}'"),
            Self::Lsb => write!(f, "'['"),
            Self::Rsb => write!(f, "']'"),

            Self::Assign => write!(f, "'='"),
            Self::Plus => write!(f, "'+'"),
            Self::AddAssign => write!(f, "'+='"),
            Self::Minus => write!(f, "'-'"),
            Self::SubAssign => write!(f, "'-='"),
            Self::Star => write!(f, "'*'"),
            Self::MulAssign => write!(f, "'*='"),
            Self::Slash => write!(f, "'/'"),
            Self::DivAssign => write!(f, "'/='"),
            Self::Percent => write!(f, "'%'"),
            Self::ModAssign => write!(f, "'%='"),
            Self::Ampersand => write!(f, "'&'"),
            Self::And => write!(f, "'&&'"),
            Self::AndAssign => write!(f, "'&='"),
            Self::Stick => write!(f, "'|'"),
            Self::Or => write!(f, "'||'"),
            Self::OrAssign => write!(f, "'|='"),
            Self::Xor => write!(f, "'^'"),
            Self::XorAssign => write!(f, "'^='"),
            Self::Comma => write!(f, "','"),
            Self::Dot => write!(f, "'.'"),
            Self::Colon => write!(f, "':'"),
            Self::Dcolon => write!(f, "'::'"),

            Self::Lt => write!(f, "'<'"),
            Self::Lte => write!(f, "'<='"),
            Self::Gt => write!(f, "'>'"),
            Self::Gte => write!(f, "'>='"),
            Self::LShift => write!(f, "'<<'"),
            Self::LShiftAssign => write!(f, "'<<='"),
            Self::RShift => write!(f, "'>>'"),
            Self::RShiftAssign => write!(f, "'>>='"),
            Self::Eq => write!(f, "'=='"),
            Self::Neq => write!(f, "'!='"),
            Self::Not => write!(f, "'!'"),

            Self::KwVar => write!(f, "var"),
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
            Self::KwUnion => write!(f, "union"),
            Self::KwVariant => write!(f, "variant"),
            Self::KwImpl => write!(f, "impl"),
            Self::KwEnum => write!(f, "enum"),
            Self::KwAlias => write!(f, "alias"),
            Self::KwDef => write!(f, "def"),
            Self::KwModule => write!(f, "module"),
            Self::KwUse => write!(f, "use"),
            Self::KwSuper => write!(f, "super"),
            Self::KwSelf => write!(f, "self"),
            Self::KwCrate => write!(f, "crate"),
            Self::KwExtern => write!(f, "extern"),
            Self::KwStatic => write!(f, "static"),
            Self::KwMut => write!(f, "mut"),
            Self::KwConst => write!(f, "const"),
            Self::KwAs => write!(f, "as"),
            Self::KwSafe => write!(f, "safe"),
            Self::KwUnsafe => write!(f, "unsafe"),
            Self::KwPub => write!(f, "pub"),

            Self::Identifier(val) => write!(f, "{val}"),
            Self::Integer(val) => write!(f, "{val}"),
            Self::Decimal(val) => write!(f, "{val}"),
            Self::Text(val) => write!(f, "{val:?}"),

            Self::Unknown => write!(f, "unknown token"),
        }
    }
}
