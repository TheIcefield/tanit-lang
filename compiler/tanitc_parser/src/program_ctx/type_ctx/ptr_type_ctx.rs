use tanitc_ast::program_ctx::type_ctx::ptr_type_ctx::PtrTypeCtx;
use tanitc_lexer::token::lexeme::Lexeme;
use tanitc_messages::Message;

use crate::{ParseResult, Parser};

impl Parser {
    pub(crate) fn parse_ptr_type_ctx(&mut self) -> ParseResult<PtrTypeCtx> {
        Ok(PtrTypeCtx {
            star_tkn: self.consume_token(Lexeme::Star)?,
            mut_tkn: {
                let next = self.peek_token().ok_or(Message::reached_eof())?;
                match next.lexeme_ref() {
                    Lexeme::KwMut | Lexeme::KwConst => {
                        self.get_token();
                        next
                    }
                    _ => {
                        return Err(Message::unexpected_token(
                            &next,
                            &[Lexeme::KwMut, Lexeme::KwConst],
                        ))
                    }
                }
            },
            type_ctx: Box::new(self.parse_type_ctx()?),
        })
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_str_eq;
    use tanitc_lexer::token::lexeme::Lexeme;

    use crate::Parser;

    #[test]
    fn parse_const_ptr_type_ctx_test() {
        const SRC_TEXT: &str = "*const hello";

        let mut parser = Parser::from_text(SRC_TEXT);
        let type_ctx = parser.parse_ptr_type_ctx().unwrap();

        assert_eq!(*type_ctx.star_tkn.lexeme_ref(), Lexeme::Star);
        assert_eq!(*type_ctx.mut_tkn.lexeme_ref(), Lexeme::KwConst);
        assert!(type_ctx.type_ctx.is_named());
    }

    #[test]
    fn parse_mut_ptr_type_ctx_test() {
        const SRC_TEXT: &str = "*mut hello";

        let mut parser = Parser::from_text(SRC_TEXT);
        let type_ctx = parser.parse_ptr_type_ctx().unwrap();

        assert_eq!(*type_ctx.star_tkn.lexeme_ref(), Lexeme::Star);
        assert_eq!(*type_ctx.mut_tkn.lexeme_ref(), Lexeme::KwMut);
        assert!(type_ctx.type_ctx.is_named());
    }

    #[test]
    fn parse_ptr_type_ctx_bad_test() {
        const SRC_TEXT: &str = "* hello";
        const EXPECTED_ERR: &str = "Unexpected token: hello. Expected: mut, const.";

        let mut parser = Parser::from_text(SRC_TEXT);
        let err = parser.parse_ptr_type_ctx().err().unwrap();

        assert_str_eq!(err.text, EXPECTED_ERR);
    }
}
