use tanitc_ast::program_ctx::statement_ctx::expression_ctx::literal_ctx::LiteralCtx;
use tanitc_lexer::token::lexeme::Lexeme;
use tanitc_messages::Message;

use crate::{ParseResult, Parser};

pub(crate) mod array_literal_ctx;
pub(crate) mod struct_literal_ctx;
pub(crate) mod tuple_literal_ctx;

impl Parser {
    pub fn parse_literal_ctx(&mut self) -> ParseResult<LiteralCtx> {
        let next = self.peek_token().ok_or(Message::reached_eof())?;

        match next.lexeme_ref() {
            lexem if lexem.is_integer() => Ok(LiteralCtx::Integer(self.consume_integer()?)),
            lexem if lexem.is_decimal() => Ok(LiteralCtx::Decimal(self.consume_decimal()?)),

            Lexeme::Lsb => self.parse_array_literal_ctx().map(LiteralCtx::Array),

            _ => Err(Message::unexpected_token(&next, &[])),
        }
    }
}
