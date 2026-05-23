use tanitc_ast::program_ctx::statement_ctx::attributes_ctx::AttributesCtx;
use tanitc_lexer::token::lexeme::Lexeme;

use crate::{ParseResult, Parser};

impl Parser {
    pub fn parse_attributes_ctx(&mut self) -> ParseResult<AttributesCtx> {
        let mut attrs = AttributesCtx::default();

        while let Some(next) = self.peek_token() {
            match next.lexeme_ref() {
                Lexeme::KwSafe => {
                    self.get_token();
                    attrs.safe_tkn = Some(next)
                }
                Lexeme::KwUnsafe => {
                    self.get_token();
                    attrs.unsafe_tkn = Some(next)
                }
                Lexeme::KwPub => {
                    self.get_token();
                    attrs.pub_tkn = Some(next);
                }
                _ => break,
            }
        }

        Ok(attrs)
    }
}

#[cfg(test)]
mod tests {
    use tanitc_lexer::token::lexeme::Lexeme;

    use crate::Parser;

    #[test]
    fn attrs_test() {
        const SRC_TEXT: &str = "unsafe pub";

        let mut parser = Parser::from_text(SRC_TEXT);
        let attrs = parser.parse_attributes_ctx().unwrap();

        assert_eq!(*attrs.pub_tkn.as_ref().unwrap().lexeme_ref(), Lexeme::KwPub);
        assert_eq!(attrs.safe_tkn, None);
        assert_eq!(
            *attrs.unsafe_tkn.as_ref().unwrap().lexeme_ref(),
            Lexeme::KwUnsafe
        );
    }

    #[test]
    fn attrs_pub_test() {
        const SRC_TEXT: &str = "pub";

        let mut parser = Parser::from_text(SRC_TEXT);
        let attrs = parser.parse_attributes_ctx().unwrap();

        assert_eq!(*attrs.pub_tkn.as_ref().unwrap().lexeme_ref(), Lexeme::KwPub);
        assert_eq!(attrs.safe_tkn, None);
        assert_eq!(attrs.unsafe_tkn, None);
    }

    #[test]
    fn attrs_safe_test() {
        const SRC_TEXT: &str = "safe";

        let mut parser = Parser::from_text(SRC_TEXT);
        let attrs = parser.parse_attributes_ctx().unwrap();

        assert_eq!(attrs.pub_tkn, None);
        assert_eq!(
            *attrs.safe_tkn.as_ref().unwrap().lexeme_ref(),
            Lexeme::KwSafe
        );
        assert_eq!(attrs.unsafe_tkn, None);
    }
}
