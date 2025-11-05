use tanitc_ast::program_ctx::{
    name_ctx::NameCtx,
    statement_ctx::expression_ctx::literal_ctx::struct_literal_ctx::{
        StructFieldLiteralCtx, StructLiteralCtx,
    },
};
use tanitc_lexer::token::{lexeme::Lexeme, Token};
use tanitc_messages::Message;

use crate::{ParseResult, Parser};

impl Parser {
    pub(crate) fn parse_struct_literal_ctx(
        &mut self,
        name_tkn: Token,
    ) -> ParseResult<StructLiteralCtx> {
        Ok(StructLiteralCtx {
            name_ctx: Box::new(NameCtx { name_tkn }),
            lcb_tkn: self.consume_token(Lexeme::Lcb)?,
            elements: self.parse_struct_literal_elements()?,
            rcb_tkn: self.consume_token(Lexeme::Rcb)?,
        })
    }

    fn parse_struct_literal_elements(
        &mut self,
    ) -> ParseResult<Vec<(Option<StructFieldLiteralCtx>, Option<Token>)>> {
        let mut fields = Vec::<(Option<StructFieldLiteralCtx>, Option<Token>)>::new();

        loop {
            let Some(next) = self.peek_token() else {
                break;
            };

            let field_ctx = match next.lexeme_ref() {
                Lexeme::Rcb => break,

                Lexeme::EndOfLine => None,

                Lexeme::Identifier(_) => Some(self.parse_struct_field_literal_ctx()?),

                _ => {
                    self.error(Message::unexpected_token(&next, &[]));
                    self.skip_until(&[Lexeme::EndOfLine]);

                    continue;
                }
            };

            let nl_tkn = self.consume_token(Lexeme::EndOfLine).ok();

            fields.push((field_ctx, nl_tkn));
        }

        Ok(fields)
    }

    fn parse_struct_field_literal_ctx(&mut self) -> ParseResult<StructFieldLiteralCtx> {
        Ok(StructFieldLiteralCtx {
            name_ctx: Box::new(self.parse_name_ctx()?),
            colon_tkn: self.consume_token(Lexeme::Colon)?,
            expression_ctx: Box::new(self.parse_expression_ctx()?),
        })
    }
}
