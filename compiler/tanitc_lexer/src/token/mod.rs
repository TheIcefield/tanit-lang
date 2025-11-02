pub mod lexem;

use super::location::Location;
pub use lexem::Lexem;
use std::fmt::Display;

#[derive(Default, Debug, Clone, PartialEq)]
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

    pub fn location_ref(&self) -> &Location {
        &self.location
    }

    pub fn location_mut(&mut self) -> &mut Location {
        &mut self.location
    }

    pub fn lexem_ref(&self) -> &Lexem {
        &self.lexem
    }

    pub fn lexem_mut(&mut self) -> &mut Lexem {
        &mut self.lexem
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]: \"{}\"", self.location, self.lexem)
    }
}
