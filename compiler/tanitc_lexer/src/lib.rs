//! Lexical analyzer (tokenizer) for the Tanit language.
//!
//! This crate converts a raw character stream into a sequence of [`Token`]s,
//! each carrying a [`Lexeme`] (the token kind) and a [`Location`] (its source
//! position). The lexer handles:
//!
//! - Single-line comments (prefixed with `#`)
//! - Whitespace skipping (spaces and tabs; newlines produce `EndOfLine` tokens)
//! - Multi-character operators (e.g. `+=`, `<<=`, `==`)
//! - Keywords (`def`, `func`, `if`, `while`, etc.)
//! - Identifiers, integer/decimal literals, and string literals
//!
//! # Key Types
//!
//! - [`Lexer`] — the main tokenizer. Wraps a `Peekable<Chars>` input stream
//!   and produces tokens on demand.
//! - [`Token`] — a lexeme paired with its source location.
//! - [`Lexeme`] — the kind of a token (operator, keyword, identifier, etc.).
//! - [`Location`] — a source position (file path, line, column).
//!
//! # Example
//!
//! ```
//! use tanitc_lexer::Lexer;
//! use std::path::PathBuf;
//!
//! let src = "var x = 42";
//! let path = PathBuf::from("example.tt");
//! let mut lexer = Lexer::new(src.chars().peekable(), &path);
//!
//! let tokens = lexer.tokenize();
//! assert_eq!(tokens.len(), 4); // var, x, =, 42
//! ```

pub mod location;
pub mod token;

use std::{
    iter::Peekable,
    path::{Path, PathBuf},
    str::Chars,
};

use location::Location;
use tanitc_ident::Ident;

use crate::token::{lexeme::Lexeme, Token};

/// A vector of tokens produced by the lexer.
pub type Tokens = Vec<Token>;

/// The Tanit language lexer.
///
/// `Lexer` wraps a `Peekable<Chars>` input stream and produces [`Token`]s
/// one at a time. It tracks the current source [`Location`] (file, line,
/// column) as it consumes characters.
///
/// # Modes
///
/// The lexer supports two tokenization modes:
///
/// - **Normal mode** — multi-character operators are recognized
///   (e.g. `+=`, `<<=`, `==`).
/// - **Singular mode** — only single-character tokens are emitted;
///   compound operators are split into their individual characters
///   (e.g. `+=` becomes `+` followed by `=`).
///
/// # Example
///
/// ```
/// use tanitc_lexer::Lexer;
/// use std::path::PathBuf;
///
/// let path = PathBuf::from("main.tt");
/// let mut lexer = Lexer::new("1 + 2".chars().peekable(), &path);
///
/// let t1 = lexer.get().unwrap();
/// assert_eq!(t1.lexeme_ref().to_string(), "1");
///
/// let t2 = lexer.get().unwrap();
/// assert_eq!(t2.lexeme_ref().to_string(), "'+'");
/// ```
pub struct Lexer<'a> {
    location: Location,
    next_token: Option<Token>,
    input: Peekable<Chars<'a>>,
    /// When `true`, each token is printed to stdout as it is produced.
    pub verbose_tokens: bool,
}

impl<'a> Lexer<'a> {
    /// Creates a new lexer from a character iterator and a file path.
    ///
    /// The `path` is used for source-location tracking in error messages.
    pub fn new(input: Peekable<Chars<'a>>, path: &Path) -> Self {
        Self {
            location: Location::new(path),
            next_token: None,
            verbose_tokens: false,
            input,
        }
    }

    /// Consumes the entire input and returns all produced tokens.
    pub fn tokenize(&mut self) -> Tokens {
        let mut tokens = Tokens::new();

        while let Some(token) = self.get() {
            tokens.push(token);
        }

        tokens
    }

    /// Returns the next token from the input stream, or `None` at EOF.
    ///
    /// Multi-character operators are recognized in this mode.
    /// If [`verbose_tokens`](Self::verbose_tokens) is enabled, the token is
    /// also printed to stdout.
    pub fn get(&mut self) -> Option<Token> {
        let tkn = self.get_next(false);

        if self.verbose_tokens {
            if let Some(tkn) = &tkn {
                println!("{tkn}");
            } else {
                println!("EOF");
            };
        }

        tkn
    }

    /// Returns the next token, using singular (single-character) mode.
    ///
    /// In this mode compound operators are not recognized; for example `+=`
    /// is split into `+` and `=`.
    pub fn get_singular(&mut self) -> Option<Token> {
        let tkn = self.get_next(true);

        if self.verbose_tokens {
            if let Some(tkn) = &tkn {
                println!("{tkn}");
            } else {
                println!("EOF");
            };
        }

        tkn
    }

    /// Peeks at the next token without consuming it.
    ///
    /// The peeked token is cached and returned by the subsequent call to
    /// [`get`](Self::get) or [`peek`](Self::peek).
    pub fn peek(&mut self) -> Option<Token> {
        if self.next_token.is_some() {
            let tkn = self.next_token.clone().unwrap();
            return Some(tkn);
        }

        self.next_token = self.get_next(false);
        self.next_token.clone()
    }

    /// Peeks at the next token in singular mode without consuming it.
    pub fn peek_singular(&mut self) -> Option<Token> {
        if self.next_token.is_some() {
            let tkn = self.next_token.clone().unwrap();
            return Some(tkn);
        }

        self.next_token = self.get_next(true);
        self.next_token.clone()
    }

    /// Returns a reference to the current source location.
    pub fn location_ref(&self) -> &Location {
        &self.location
    }

    /// Returns a mutable reference to the current source location.
    pub fn location_mut(&mut self) -> &mut Location {
        &mut self.location
    }

    /// Returns the file path associated with this lexer's input.
    ///
    /// # Panics
    ///
    /// Panics if the lexer was not created with a file path (i.e. the path is empty).
    pub fn get_path(&self) -> PathBuf {
        if self.location.path.as_path_buf().as_os_str().is_empty() {
            panic!("Lexer input stream is not a file");
        }

        self.location.path.as_path_buf()
    }

    fn peek_char(&mut self) -> Option<&char> {
        self.input.peek()
    }

    fn next_char(&mut self) -> Option<char> {
        let next = self.input.next();

        if next.is_some_and(|ch| ch == '\n') {
            self.location.new_line();
        } else if next.is_some() {
            self.location.shift();
        }

        next
    }

    fn skip_spaces(&mut self) {
        while self.peek_char().is_some_and(|ch| ch.is_ascii_whitespace()) {
            if self.peek_char().is_some_and(|ch| *ch == '\n') {
                self.location.new_line();
                return;
            }

            self.next_char();
        }
    }

    fn skip_comment(&mut self) {
        let Some(ch) = self.peek_char() else {
            return;
        };

        if *ch == '#' {
            while self.next_char() != Some('\n') {}

            self.location.new_line();
        }
    }

    fn get_next(&mut self, singular: bool) -> Option<Token> {
        if self.next_token.is_some() {
            let token = self.next_token.clone().unwrap();
            self.next_token = None;
            return Some(token);
        }

        loop {
            self.skip_spaces();
            self.skip_comment();

            let ch = self.peek_char();

            if ch.is_some_and(|ch| *ch != '#' && !ch.is_ascii_whitespace() || *ch == '\n')
                || ch.is_none()
            {
                break;
            }
        }

        let next_char = self.peek_char()?;

        let lexem = match next_char {
            '\n' => {
                self.next_char();
                Lexeme::EndOfLine
            }
            '(' => {
                self.next_char();
                Lexeme::LParen
            }
            ')' => {
                self.next_char();
                Lexeme::RParen
            }
            '[' => {
                self.next_char();
                Lexeme::Lsb
            }
            ']' => {
                self.next_char();
                Lexeme::Rsb
            }
            '{' => {
                self.next_char();
                Lexeme::Lcb
            }
            '}' => {
                self.next_char();
                Lexeme::Rcb
            }
            '.' => {
                self.next_char();
                Lexeme::Dot
            }
            ',' => {
                self.next_char();
                Lexeme::Comma
            }
            '>' => {
                self.next_char();
                let mut lexeme = Lexeme::Gt;

                if self.peek_char().is_some_and(|ch| *ch == '=') && !singular {
                    self.next_char();
                    lexeme = Lexeme::Gte;
                } else if self.peek_char().is_some_and(|ch| *ch == '>') && !singular {
                    self.next_char();
                    lexeme = Lexeme::RShift;

                    if self.peek_char().is_some_and(|ch| *ch == '=') {
                        self.next_char();
                        lexeme = Lexeme::RShiftAssign;
                    }
                }

                lexeme
            }
            '<' => {
                self.next_char();
                let mut lexeme = Lexeme::Lt;

                if self.peek_char().is_some_and(|ch| *ch == '=') && !singular {
                    self.next_char();
                    lexeme = Lexeme::Lte;
                } else if self.peek_char().is_some_and(|ch| *ch == '<') && !singular {
                    self.next_char();
                    lexeme = Lexeme::LShift;

                    if self.peek_char().is_some_and(|ch| *ch == '=') {
                        self.next_char();
                        lexeme = Lexeme::LShiftAssign;
                    }
                }

                lexeme
            }
            '+' => {
                self.next_char();
                let mut lexeme = Lexeme::Plus;

                if self.peek_char().is_some_and(|ch| *ch == '=') && !singular {
                    self.next_char();
                    lexeme = Lexeme::AddAssign;
                }

                lexeme
            }
            '-' => {
                self.next_char();
                let mut lexeme = Lexeme::Minus;

                if self.peek_char().is_some_and(|ch| *ch == '=') && !singular {
                    self.next_char();
                    lexeme = Lexeme::SubAssign;
                }

                lexeme
            }
            '/' => {
                self.next_char();
                let mut lexeme = Lexeme::Slash;

                if self.peek_char().is_some_and(|ch| *ch == '=') && !singular {
                    self.next_char();
                    lexeme = Lexeme::DivAssign;
                }

                lexeme
            }
            '%' => {
                self.next_char();
                let mut lexeme = Lexeme::Percent;

                if self.peek_char().is_some_and(|ch| *ch == '=') && !singular {
                    self.next_char();
                    lexeme = Lexeme::ModAssign;
                }

                lexeme
            }
            '*' => {
                self.next_char();
                let mut lexeme = Lexeme::Star;

                if self.peek_char().is_some_and(|ch| *ch == '=') && !singular {
                    self.next_char();
                    lexeme = Lexeme::MulAssign;
                }

                lexeme
            }
            '!' => {
                self.next_char();
                let mut lexeme = Lexeme::Not;

                if self.peek_char().is_some_and(|ch| *ch == '=') && !singular {
                    self.next_char();
                    lexeme = Lexeme::Neq;
                }

                lexeme
            }
            '=' => {
                self.next_char();
                let mut lexeme = Lexeme::Assign;

                if self.peek_char().is_some_and(|ch| *ch == '=') && !singular {
                    self.next_char();
                    lexeme = Lexeme::Eq;
                }

                lexeme
            }
            '&' => {
                self.next_char();
                let mut lexeme = Lexeme::Ampersand;

                if self.peek_char().is_some_and(|ch| *ch == '&') && !singular {
                    self.next_char();
                    lexeme = Lexeme::And;
                } else if self.peek_char().is_some_and(|ch| *ch == '=') && !singular {
                    self.next_char();
                    lexeme = Lexeme::AndAssign;
                }

                lexeme
            }
            '^' => {
                self.next_char();
                let mut lexeme = Lexeme::Xor;

                if self.peek_char().is_some_and(|ch| *ch == '=') && !singular {
                    self.next_char();
                    lexeme = Lexeme::XorAssign;
                }

                lexeme
            }
            '|' => {
                self.next_char();
                let mut lexeme = Lexeme::Stick;

                if self.peek_char().is_some_and(|ch| *ch == '=') && !singular {
                    self.next_char();
                    lexeme = Lexeme::OrAssign;
                } else if self.peek_char().is_some_and(|ch| *ch == '|') && !singular {
                    self.next_char();
                    lexeme = Lexeme::Or;
                }

                lexeme
            }
            ':' => {
                self.next_char();
                let mut lexeme = Lexeme::Colon;

                if self.peek_char().is_some_and(|ch| *ch == ':') && !singular {
                    self.next_char();
                    lexeme = Lexeme::Dcolon;
                }

                lexeme
            }

            _ if '\"' == *next_char => self.get_text_lexem()?,
            _ if next_char.is_ascii_digit() => self.get_numeric_lexem()?,
            _ if next_char.is_ascii_alphabetic() || '_' == *next_char => self.get_string_lexem()?,
            _ => Lexeme::Unknown,
        };

        Some(Token::new(lexem, *self.location_ref()))
    }

    fn get_numeric_lexem(&mut self) -> Option<Lexeme> {
        let mut text = String::new();
        let mut is_float = false;

        while self
            .peek_char()
            .is_some_and(|ch| ch.is_ascii_digit() || *ch == '.')
        {
            if self.peek_char().is_some_and(|ch| *ch == '.') {
                if is_float {
                    return None;
                }
                is_float = true;
            }

            text.push(self.next_char()?);
        }

        Some(if is_float {
            Lexeme::Decimal(text)
        } else {
            Lexeme::Integer(text)
        })
    }

    fn get_string_lexem(&mut self) -> Option<Lexeme> {
        let mut text = String::new();

        while self
            .peek_char()
            .is_some_and(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
        {
            text.push(self.next_char()?);
        }

        Some(match &text[..] {
            "def" => Lexeme::KwDef,
            "module" => Lexeme::KwModule,
            "struct" => Lexeme::KwStruct,
            "union" => Lexeme::KwUnion,
            "variant" => Lexeme::KwVariant,
            "impl" => Lexeme::KwImpl,
            "enum" => Lexeme::KwEnum,
            "var" => Lexeme::KwVar,
            "mut" => Lexeme::KwMut,
            "const" => Lexeme::KwConst,
            "alias" => Lexeme::KwAlias,
            "func" => Lexeme::KwFunc,
            "if" => Lexeme::KwIf,
            "else" => Lexeme::KwElse,
            "loop" => Lexeme::KwLoop,
            "do" => Lexeme::KwDo,
            "while" => Lexeme::KwWhile,
            "for" => Lexeme::KwFor,
            "continue" => Lexeme::KwContinue,
            "break" => Lexeme::KwBreak,
            "return" => Lexeme::KwReturn,
            "extern" => Lexeme::KwExtern,
            "static" => Lexeme::KwStatic,
            "use" => Lexeme::KwUse,
            "super" => Lexeme::KwSuper,
            "self" => Lexeme::KwSelf,
            "crate" => Lexeme::KwCrate,
            "as" => Lexeme::KwAs,
            "safe" => Lexeme::KwSafe,
            "unsafe" => Lexeme::KwUnsafe,
            "pub" => Lexeme::KwPub,
            _ => Lexeme::Identifier(Ident::from(text)),
        })
    }

    fn get_text_lexem(&mut self) -> Option<Lexeme> {
        let mut text = String::new();

        // push opening '\"'
        text.push(self.next_char()?);

        while self.peek_char().is_some_and(|ch| ch.is_ascii()) {
            if self.peek_char().is_some_and(|ch| *ch == '\"') {
                // push closing '\"'
                text.push(self.next_char()?);
                break;
            }

            // push char
            text.push(self.next_char()?);
        }

        Some(Lexeme::Text(text))
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{location::Location, token::lexeme::Lexeme, Lexer};

    #[test]
    fn lexer_test() {
        const SRC_TEXT: &str = "+ hello func var 65 -= <<\n struct alpha safe unsafe impl";

        let test_path = PathBuf::from("test");

        let mut lexer = Lexer::new(SRC_TEXT.chars().peekable(), &test_path);

        let mut tkn = lexer.get().unwrap();
        let mut location = Location::new(&test_path);

        location.col = 1;
        assert_eq!(*tkn.lexeme_ref(), Lexeme::Plus);
        assert_eq!(tkn.get_location(), location);

        tkn = lexer.get().unwrap();
        location.col = 7;
        assert_eq!(*tkn.lexeme_ref(), Lexeme::identifier("hello"));
        assert_eq!(tkn.get_location(), location);

        tkn = lexer.get().unwrap();
        location.col = 12;
        assert_eq!(*tkn.lexeme_ref(), Lexeme::KwFunc);
        assert_eq!(tkn.get_location(), location);

        tkn = lexer.get().unwrap();
        location.col = 16;
        assert_eq!(*tkn.lexeme_ref(), Lexeme::KwVar);
        assert_eq!(tkn.get_location(), location);

        tkn = lexer.get().unwrap();
        location.col = 19;
        assert_eq!(*tkn.lexeme_ref(), Lexeme::integer(65));
        assert_eq!(tkn.get_location(), location);

        tkn = lexer.get().unwrap();
        location.col = 22;
        assert_eq!(*tkn.lexeme_ref(), Lexeme::SubAssign);
        assert_eq!(tkn.get_location(), location);

        tkn = lexer.get().unwrap();
        location.col = 25;
        assert_eq!(*tkn.lexeme_ref(), Lexeme::LShift);
        assert_eq!(tkn.get_location(), location);

        tkn = lexer.get().unwrap();
        location.row = 2;
        location.col = 0;
        assert_eq!(*tkn.lexeme_ref(), Lexeme::EndOfLine);
        assert_eq!(tkn.get_location(), location);

        tkn = lexer.get().unwrap();
        location.col = 7;
        assert_eq!(*tkn.lexeme_ref(), Lexeme::KwStruct);
        assert_eq!(tkn.get_location(), location);

        tkn = lexer.get().unwrap();
        location.col = 13;
        assert_eq!(*tkn.lexeme_ref(), Lexeme::identifier("alpha"));
        assert_eq!(tkn.get_location(), location);

        tkn = lexer.get().unwrap();
        location.col = 18;
        assert_eq!(*tkn.lexeme_ref(), Lexeme::KwSafe);
        assert_eq!(tkn.get_location(), location);

        tkn = lexer.get().unwrap();
        location.col = 25;
        assert_eq!(*tkn.lexeme_ref(), Lexeme::KwUnsafe);
        assert_eq!(tkn.get_location(), location);

        tkn = lexer.get().unwrap();
        location.col = 30;
        assert_eq!(*tkn.lexeme_ref(), Lexeme::KwImpl);
        assert_eq!(tkn.get_location(), location);

        assert_eq!(lexer.get(), None);
    }
}
