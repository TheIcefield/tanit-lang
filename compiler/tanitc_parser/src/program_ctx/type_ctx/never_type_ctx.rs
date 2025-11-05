use tanitc_ast::program_ctx::type_ctx::never_type_ctx::NeverTypeCtx;
use tanitc_lexer::token::lexeme::Lexeme;

use crate::{ParseResult, Parser};

impl Parser {
    pub(crate) fn parse_never_type_ctx(&mut self) -> ParseResult<NeverTypeCtx> {
        Ok(NeverTypeCtx {
            excm_tkn: self.consume_token(Lexeme::Not)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use tanitc_lexer::token::lexeme::Lexeme;

    use crate::Parser;

    #[test]
    fn parse_ptr_type_ctx_bad_test() {
        const SRC_TEXT: &str = "!";

        let mut parser = Parser::from_text(SRC_TEXT);
        let type_ctx = parser.parse_never_type_ctx().unwrap();

        assert_eq!(*type_ctx.excm_tkn.lexeme_ref(), Lexeme::Not);
    }
}
