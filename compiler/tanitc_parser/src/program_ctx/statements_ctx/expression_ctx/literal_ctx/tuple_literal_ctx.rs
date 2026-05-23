use tanitc_ast::program_ctx::statement_ctx::expression_ctx::ExpressionCtx;
use tanitc_lexer::token::{lexeme::Lexeme, Token};
use tanitc_messages::Message;

use crate::{ParseResult, Parser};

impl Parser {
    pub fn parse_tuple_literal_elements_ctx(
        &mut self,
    ) -> ParseResult<Vec<(Option<ExpressionCtx>, Option<Token>)>> {
        let mut elements = Vec::<(Option<ExpressionCtx>, Option<Token>)>::new();

        while let Some(next) = self.peek_token() {
            let element = match next.lexeme_ref() {
                Lexeme::Rcb => break,

                Lexeme::EndOfLine => None,

                Lexeme::Identifier(_)
                | Lexeme::Integer(_)
                | Lexeme::Decimal(_)
                | Lexeme::Ampersand
                | Lexeme::Plus
                | Lexeme::Minus
                | Lexeme::Star
                | Lexeme::Not
                | Lexeme::LParen => Some(self.parse_expression_ctx()?),

                _ => {
                    self.error(Message::unexpected_token(&next, &[]));
                    self.skip_until(&[Lexeme::EndOfLine]);

                    continue;
                }
            };

            let nl_tkn = self.consume_token(Lexeme::EndOfLine).ok();

            elements.push((element, nl_tkn));
        }

        Ok(elements)
    }
}

/*
#[cfg(test)]
mod tests {
    use crate::Parser;


    #[test]
    fn tuple_parse_test() {
        // Given
        const SRC_TEXT: &str = "\nfunc main() {\
                                \n    var t = (1.0, 2, 3.0)\
                                \n}";

        let mut parser = Parser::from_text(SRC_TEXT);

        // When
        let ast = parser.parse_func_def_ctx().unwrap();

        // Then
    }
}
*/
