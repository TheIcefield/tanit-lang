use std::{
    char::ParseCharError,
    num::{ParseFloatError, ParseIntError},
    str::ParseBoolError,
};

use crate::parser::{
    location::Location,
    token::{Lexem, Token},
};

#[derive(Default, Debug, Clone)]
pub struct Message {
    pub location: Location,
    pub text: String,
}

pub type Error = Message;
pub type Warning = Message;

pub type Errors = Vec<Error>;
pub type Warnings = Vec<Warning>;

impl Message {
    pub fn new(location: Location, text: &str) -> Self {
        Self {
            location,
            text: text.to_string(),
        }
    }

    pub fn unexpected_token(token: Token, expected: &[Lexem]) -> Self {
        let mut text = format!("Unexpected token: {}. ", token.lexem);

        if !expected.is_empty() {
            text.push_str(&format!("Expected: {}", expected[0]));

            for lexem in expected.iter().skip(1) {
                text.push_str(&format!(", {}", lexem));
            }

            text.push('.');
        }

        Self {
            location: token.location,
            text,
        }
    }

    pub fn multiple_ids(location: Location, id: &str) -> Self {
        Self {
            location,
            text: format!("Identifier \"{}\" defined multiple times", id),
        }
    }

    pub fn parse_int_error(location: Location, err: ParseIntError) -> Self {
        Self {
            location,
            text: err.to_string(),
        }
    }

    pub fn parse_float_error(location: Location, err: ParseFloatError) -> Self {
        Self {
            location,
            text: err.to_string(),
        }
    }

    pub fn parse_char_error(location: Location, err: ParseCharError) -> Self {
        Self {
            location,
            text: err.to_string(),
        }
    }

    pub fn parse_bool_error(location: Location, err: ParseBoolError) -> Self {
        Self {
            location,
            text: err.to_string(),
        }
    }
}
