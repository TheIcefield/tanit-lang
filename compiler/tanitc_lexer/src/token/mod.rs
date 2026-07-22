//! Token representation.

use crate::{location::Location, token::lexeme::Lexeme};
use std::fmt::Display;
use tanitc_ident::Ident;

pub mod lexeme;

/// A lexer token — a [`Lexeme`] paired with its source [`Location`].
///
/// # Example
///
/// ```
/// use tanitc_lexer::token::{Token, lexeme::Lexeme};
/// use tanitc_lexer::location::Location;
/// use std::path::PathBuf;
///
/// let loc = Location::new(&PathBuf::from("test.tt"));
/// let token = Token::new(Lexeme::KwVar, loc);
///
/// assert!(token.is_identifier() == false);
/// assert_eq!(token.get_location().to_string(), "test.tt:1:1");
/// ```
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Token {
    lexeme: Lexeme,
    location: Location,
}

impl Token {
    /// Creates a new token with the given lexeme and source location.
    pub fn new(lexeme: Lexeme, location: Location) -> Self {
        Self { lexeme, location }
    }

    /// Returns `true` if this token is an identifier.
    pub fn is_identifier(&self) -> bool {
        matches!(self.lexeme, Lexeme::Identifier(_))
    }

    /// Returns `true` if this token is an integer literal.
    pub fn is_integer(&self) -> bool {
        matches!(self.lexeme, Lexeme::Integer(_))
    }

    /// Returns `true` if this token is a decimal (floating-point) literal.
    pub fn is_decimal(&self) -> bool {
        matches!(self.lexeme, Lexeme::Decimal(_))
    }

    /// Returns the source location of this token.
    pub fn get_location(&self) -> Location {
        self.location
    }

    /// Returns a reference to this token's lexeme.
    pub fn lexeme_ref(&self) -> &Lexeme {
        &self.lexeme
    }

    /// Returns a mutable reference to this token's lexeme.
    pub fn lexeme_mut(&mut self) -> &mut Lexeme {
        &mut self.lexeme
    }

    /// Extracts the [`Ident`] from an identifier token.
    ///
    /// # Panics
    ///
    /// Panics if the token is not an identifier.
    pub fn identifier(&self) -> Ident {
        if let Lexeme::Identifier(id) = &self.lexeme {
            *id
        } else {
            panic!("Token is not an identifier")
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]: \"{}\"", self.location, self.lexeme)
    }
}
