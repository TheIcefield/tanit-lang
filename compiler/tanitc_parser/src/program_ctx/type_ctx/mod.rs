use tanitc_ast::program_ctx::type_ctx::TypeCtx;
use tanitc_lexer::token::lexeme::Lexeme;
use tanitc_messages::Message;

use crate::{ParseResult, Parser};

pub(crate) mod array_type_ctx;
pub(crate) mod func_type_ctx;
pub(crate) mod named_type_ctx;
pub(crate) mod never_type_ctx;
pub(crate) mod ptr_type_ctx;
pub(crate) mod ref_type_ctx;
pub(crate) mod tuple_type_ctx;

impl Parser {
    pub fn parse_type_ctx(&mut self) -> ParseResult<TypeCtx> {
        let next = self.peek_token().ok_or(Message::reached_eof())?;

        match next.lexeme_ref() {
            Lexeme::Identifier(_) => self.parse_named_type_ctx().map(TypeCtx::Named),
            Lexeme::KwFunc => self.parse_func_type_ctx().map(TypeCtx::Func),
            Lexeme::Ampersand => self.parse_ref_type_ctx().map(TypeCtx::Ref),
            Lexeme::Star => self.parse_ptr_type_ctx().map(TypeCtx::Ptr),
            Lexeme::LParen => self.parse_tuple_type_ctx().map(TypeCtx::Tuple),
            Lexeme::Lsb => self.parse_array_type_ctx().map(TypeCtx::Array),
            Lexeme::Not => self.parse_never_type_ctx().map(TypeCtx::Never),
            _ => Err(Message::unexpected_token(&next, &[])),
        }
    }
}
