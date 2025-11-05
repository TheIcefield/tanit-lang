use tanitc_ast::program_ctx::type_ctx::ref_type_ctx::RefTypeCtx;
use tanitc_lexer::token::lexeme::Lexeme;

use crate::{ParseResult, Parser};

impl Parser {
    pub fn parse_ref_type_ctx(&mut self) -> ParseResult<RefTypeCtx> {
        Ok(RefTypeCtx {
            ampersand_tkn: self.consume_token(Lexeme::Ampersand)?,
            mut_tkn: self.consume_token(Lexeme::KwMut).ok(),
            type_ctx: Box::new(self.parse_type_ctx()?),
        })
    }
}

#[cfg(test)]
mod tests {
    use tanitc_lexer::token::lexeme::Lexeme;

    use crate::Parser;

    #[test]
    fn parse_const_ref_type_ctx_test() {
        const SRC_TEXT: &str = "&hello";

        let mut parser = Parser::from_text(SRC_TEXT);
        let type_ctx = parser.parse_ref_type_ctx().unwrap();

        assert_eq!(*type_ctx.ampersand_tkn.lexeme_ref(), Lexeme::Ampersand);
        assert!(type_ctx.mut_tkn.is_none());
        assert!(type_ctx.type_ctx.is_named());
    }

    #[test]
    fn parse_mut_ref_type_ctx_test() {
        const SRC_TEXT: &str = "&mut hello";

        let mut parser = Parser::from_text(SRC_TEXT);
        let type_ctx = parser.parse_ref_type_ctx().unwrap();

        assert_eq!(*type_ctx.ampersand_tkn.lexeme_ref(), Lexeme::Ampersand);
        assert!(type_ctx
            .mut_tkn
            .as_ref()
            .is_some_and(|tkn| *tkn.lexeme_ref() == Lexeme::KwMut));
        assert!(type_ctx.type_ctx.is_named());
    }
}
