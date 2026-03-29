use tanitc_attributes::{Mutability, Publicity, Safety};
use tanitc_lexer::token::{lexeme::Lexeme, Token};
use tanitc_messages::Message;

use crate::{AstLowResult, AstLowering};

impl AstLowering {
    pub(crate) fn low_publicity_token(&self, tkn: &Option<Token>) -> Publicity {
        let Some(tkn) = tkn else {
            return Publicity::default();
        };

        if *tkn.lexeme_ref() == Lexeme::KwPub {
            Publicity::Public
        } else {
            Publicity::Private
        }
    }

    pub(crate) fn low_mut_token(&self, tkn: &Option<Token>) -> Mutability {
        let Some(tkn) = tkn else {
            return Mutability::default();
        };

        if *tkn.lexeme_ref() == Lexeme::KwMut {
            Mutability::Mutable
        } else {
            Mutability::Immutable
        }
    }

    pub(crate) fn low_safe_token(&self, tkn: &Option<Token>) -> Safety {
        let Some(tkn) = tkn else {
            return Safety::default();
        };

        if *tkn.lexeme_ref() == Lexeme::KwSafe {
            Safety::Safe
        } else {
            Safety::Inherited
        }
    }

    pub(crate) fn low_unsafe_token(&self, tkn: &Option<Token>) -> Safety {
        let Some(tkn) = tkn else {
            return Safety::default();
        };

        if *tkn.lexeme_ref() == Lexeme::KwUnsafe {
            Safety::Unsafe
        } else {
            Safety::Inherited
        }
    }

    pub(crate) fn low_safety(
        &self,
        safe_tkn: &Option<Token>,
        unsafe_tkn: &Option<Token>,
    ) -> AstLowResult<Safety> {
        match (safe_tkn, unsafe_tkn) {
            (Some(_), None) => Ok(self.low_safe_token(safe_tkn)),
            (None, Some(_)) => Ok(self.low_unsafe_token(unsafe_tkn)),
            (None, None) => Ok(Safety::Inherited),
            (Some(safe_tkn), Some(_)) => Err(Message::new(
                safe_tkn.get_location(),
                "Used both \"safe\" and \"unsafe\" attributes. Only one is allowed",
            )),
        }
    }

    pub(crate) fn expect_incompatible_attribute(&self, tkn: &Option<Token>) -> AstLowResult<()> {
        if let Some(tkn) = tkn {
            return Err(Message::new(
                tkn.get_location(),
                format!("Incorrect attribute \"{}\"", tkn.lexeme_ref()),
            ));
        }

        Ok(())
    }
}
