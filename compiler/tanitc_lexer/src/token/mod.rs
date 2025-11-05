use crate::{location::Location, token::lexeme::Lexeme};
use std::fmt::Display;
use tanitc_ident::Ident;

pub mod lexeme;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Token {
    lexeme: Lexeme,
    location: Location,
}

impl Token {
    pub fn new(lexeme: Lexeme, location: Location) -> Self {
        Self { lexeme, location }
    }

    pub fn is_identifier(&self) -> bool {
        matches!(self.lexeme, Lexeme::Identifier(_))
    }

    pub fn is_integer(&self) -> bool {
        matches!(self.lexeme, Lexeme::Integer(_))
    }

    pub fn is_decimal(&self) -> bool {
        matches!(self.lexeme, Lexeme::Decimal(_))
    }

    pub fn get_location(&self) -> Location {
        self.location
    }

    pub fn lexeme_ref(&self) -> &Lexeme {
        &self.lexeme
    }

    pub fn lexeme_mut(&mut self) -> &mut Lexeme {
        &mut self.lexeme
    }

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
