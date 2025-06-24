use tanitc_ident::Ident;
use tanitc_lexer::{
    location::Location,
    token::{Lexem, Token},
};

use std::{
    char::ParseCharError,
    fmt::Display,
    num::{ParseFloatError, ParseIntError},
    str::ParseBoolError,
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

    pub fn from_string(location: Location, text: String) -> Self {
        Self { location, text }
    }

    pub fn unexpected_token(token: Token, expected: &[Lexem]) -> Self {
        let mut text = format!("Unexpected token: {}. ", token.lexem);

        if !expected.is_empty() {
            text.push_str(&format!("Expected: {}", expected[0]));

            for lexem in expected.iter().skip(1) {
                text.push_str(&format!(", {lexem}"));
            }

            text.push('.');
        }

        Self {
            location: token.location,
            text,
        }
    }

    pub fn multiple_ids(location: Location, id: Ident) -> Self {
        let id: String = id.into();
        Self {
            location,
            text: format!("Identifier \"{id}\" defined multiple times"),
        }
    }

    pub fn undefined_id(location: Location, id: Ident) -> Self {
        Self {
            location,
            text: format!("Undefined name \"{id}\""),
        }
    }

    pub fn undefined_variable(location: Location, var_name: Ident) -> Self {
        Self {
            location,
            text: format!("No variable \"{var_name}\" found"),
        }
    }

    pub fn undefined_func(location: Location, func_name: Ident) -> Self {
        Self {
            location,
            text: format!("No function \"{func_name}\" found"),
        }
    }

    pub fn undefined_struct(location: Location, struct_name: Ident) -> Self {
        Self {
            location,
            text: format!("No struct \"{struct_name}\" found"),
        }
    }

    pub fn undefined_union(location: Location, union_name: Ident) -> Self {
        Self {
            location,
            text: format!("No struct \"{union_name}\" found"),
        }
    }

    pub fn const_mutation(location: Location, s: &str) -> Self {
        Self {
            location,
            text: format!(
                "Cannot mutate immutable object of type \"{s}\" is immutable in current scope"
            ),
        }
    }

    pub fn const_var_mutation(location: Location, var_name: Ident) -> Self {
        Self {
            location,
            text: format!("Variable \"{var_name}\" is immutable in current scope"),
        }
    }

    pub fn const_ref_mutation(location: Location, var_name: Ident) -> Self {
        Self {
            location,
            text: format!("Reference \"{var_name}\" is immutable in current scope"),
        }
    }

    pub fn no_id_in_namespace(location: Location, namespace: Ident, id: Ident) -> Self {
        let id: String = id.into();
        Self {
            location,
            text: format!("No object named \"{id}\" in namespace {namespace}"),
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

    pub fn unreachable(location: Location, msg: String) -> Self {
        Self {
            location,
            text: format!("Compiler reached unreachable code: {msg}"),
        }
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.location, self.text)?;

        Ok(())
    }
}

pub fn print_messages(messages: &[Message]) {
    for msg in messages.iter() {
        eprintln!("{msg}");
    }
}
