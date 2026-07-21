use tanitc_ast::program_ctx::name_ctx::{NameCtx, NameSpecCtx, NameSpecSegmentCtx};
use tanitc_lexer::token::lexeme::Lexeme;

use crate::{ParseResult, Parser};

impl Parser {
    pub fn parse_name_ctx(&mut self) -> ParseResult<NameCtx> {
        Ok(NameCtx {
            name_tkn: self.consume_identifier()?,
        })
    }

    pub fn parse_name_spec_ctx(&mut self) -> ParseResult<NameSpecCtx> {
        let old = self.does_ignore_nl();
        self.set_ignore_nl_option(false);

        let names = self.parse_name_spec_segments()?;

        self.set_ignore_nl_option(old);

        Ok(NameSpecCtx { names })
    }

    fn parse_name_spec_segments(&mut self) -> ParseResult<Vec<NameSpecSegmentCtx>> {
        let mut names = Vec::<NameSpecSegmentCtx>::new();

        while self
            .peek_token()
            .is_some_and(|tkn| tkn.lexeme_ref().is_identifier())
        {
            let id = self.consume_identifier()?;

            match self.peek_token() {
                Some(token) if *token.lexeme_ref() == Lexeme::Dcolon => {
                    self.get_token();
                    names.push((id, Some(token)));
                }

                _ => {
                    names.push((id, None));
                    break;
                }
            }
        }

        Ok(names)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn good_parse_name_spec_test() {
        // Given
        const SRC_TEXT: &str = "hello::name::specifiers\n";
        let mut parser = Parser::from_text(SRC_TEXT);

        // When
        let name = parser.parse_name_spec_ctx().unwrap();

        // Then
        assert_eq!(name.names.len(), 3);

        assert_eq!(name.names[0].0.identifier().to_string(), "hello");
        assert_eq!(name.names[1].0.identifier().to_string(), "name");
        assert_eq!(name.names[2].0.identifier().to_string(), "specifiers");

        assert_ne!(name.names[0].1, None);
        assert_ne!(name.names[1].1, None);
        assert_eq!(name.names[2].1, None);
    }
}
