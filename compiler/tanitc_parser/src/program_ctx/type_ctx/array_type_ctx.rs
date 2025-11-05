use tanitc_ast::program_ctx::type_ctx::array_type_ctx::{ArrayTypeCtx, ArrayTypeLengthCtx};
use tanitc_lexer::token::lexeme::Lexeme;

use crate::{ParseResult, Parser};

impl Parser {
    pub(crate) fn parse_array_type_ctx(&mut self) -> ParseResult<ArrayTypeCtx> {
        Ok(ArrayTypeCtx {
            lsb_tkn: self.consume_token(Lexeme::Lsb)?,
            type_ctx: Box::new(self.parse_type_ctx()?),
            length_ctx: self.parse_array_type_length_ctx()?,
            rsb_tkn: self.consume_token(Lexeme::Rsb)?,
        })
    }

    fn parse_array_type_length_ctx(&mut self) -> ParseResult<Option<ArrayTypeLengthCtx>> {
        if self.is_next(Lexeme::Colon) {
            Ok(Some(ArrayTypeLengthCtx {
                colon_tkn: self.consume_token(Lexeme::Colon)?,
                expression_ctx: Box::new(self.parse_expression_ctx()?),
            }))
        } else {
            Ok(None)
        }
    }
}
