use tanitc_ast::program_ctx::statement_ctx::use_ctx::UseCtx;
use tanitc_lexer::token::lexeme::Lexeme;

use crate::{ParseResult, Parser};

impl Parser {
    pub(crate) fn parse_use_ctx(&mut self) -> ParseResult<UseCtx> {
        let use_tkn = self.consume_token(Lexeme::KwUse)?;
        let name_spec_ctx = self.parse_name_spec_ctx()?;

        Ok(UseCtx {
            use_tkn,
            name_spec_ctx,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn good_parse_use_test() {
        // Given
        const SRC_TEXT: &str = "use hello::name::specifiers\n";
        let mut parser = Parser::from_text(SRC_TEXT);

        // When
        let use_ctx = parser.parse_use_ctx().unwrap();
        let ids = &use_ctx.name_spec_ctx.names;

        // Then
        assert_eq!(*use_ctx.use_tkn.lexeme_ref(), Lexeme::KwUse);

        assert_eq!(ids.len(), 3);

        assert_eq!(ids[0].0.identifier().to_string(), "hello");
        assert_eq!(ids[1].0.identifier().to_string(), "name");
        assert_eq!(ids[2].0.identifier().to_string(), "specifiers");

        assert_ne!(ids[0].1, None);
        assert_ne!(ids[1].1, None);
        assert_eq!(ids[2].1, None);
    }
}
