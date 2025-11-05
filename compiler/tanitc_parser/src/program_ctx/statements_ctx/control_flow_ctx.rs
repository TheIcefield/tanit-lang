use tanitc_ast::program_ctx::statement_ctx::control_flow_ctx::{
    break_ctx::BreakCtx, continue_ctx::ContinueCtx, return_ctx::ReturnCtx, ControlFlowCtx,
};
use tanitc_lexer::token::lexeme::Lexeme;
use tanitc_messages::Message;

use crate::{ParseResult, Parser};

impl Parser {
    pub fn parse_control_flow_ctx(&mut self) -> ParseResult<ControlFlowCtx> {
        let next = self.peek_token().ok_or(Message::reached_eof())?;

        match next.lexeme_ref() {
            Lexeme::KwBreak => self.parse_break_ctx().map(ControlFlowCtx::Break),
            Lexeme::KwContinue => self.parse_continue_ctx().map(ControlFlowCtx::Continue),
            Lexeme::KwReturn => self.parse_return_ctx().map(ControlFlowCtx::Return),
            _ => Err(Message::unexpected_token(
                &next,
                &[Lexeme::KwBreak, Lexeme::KwContinue, Lexeme::KwReturn],
            )),
        }
    }

    fn parse_break_ctx(&mut self) -> ParseResult<BreakCtx> {
        Ok(BreakCtx {
            break_tkn: self.consume_token(Lexeme::KwBreak)?,
        })
    }

    fn parse_continue_ctx(&mut self) -> ParseResult<ContinueCtx> {
        Ok(ContinueCtx {
            continue_tkn: self.consume_token(Lexeme::KwContinue)?,
        })
    }

    fn parse_return_ctx(&mut self) -> ParseResult<ReturnCtx> {
        Ok(ReturnCtx {
            return_tkn: self.consume_token(Lexeme::KwReturn)?,
            return_expression_ctx: {
                let old_opt = self.does_ignore_nl();
                self.set_ignore_nl_option(false);

                let expr = if self.is_next(Lexeme::EndOfLine) {
                    None
                } else {
                    Some(self.parse_expression_ctx())
                };

                self.set_ignore_nl_option(old_opt);

                if let Some(expr) = expr {
                    Some(Box::new(expr?))
                } else {
                    None
                }
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use tanitc_lexer::token::lexeme::Lexeme;

    use crate::Parser;

    #[test]
    fn parse_break_test() {
        // Given
        const SRC_TEXT: &str = "break\n";
        let mut parser = Parser::from_text(SRC_TEXT);

        // When
        let break_ctx = parser.parse_break_ctx().unwrap();

        // Then
        assert_eq!(*break_ctx.break_tkn.lexeme_ref(), Lexeme::KwBreak);
    }

    #[test]
    fn parse_continue_test() {
        // Given
        const SRC_TEXT: &str = "continue\n";
        let mut parser = Parser::from_text(SRC_TEXT);

        // When
        let break_ctx = parser.parse_continue_ctx().unwrap();

        // Then
        assert_eq!(*break_ctx.continue_tkn.lexeme_ref(), Lexeme::KwContinue);
    }

    #[test]
    fn parse_return_test() {
        // Given
        const SRC_TEXT: &str = "return\n";
        let mut parser = Parser::from_text(SRC_TEXT);

        // When
        let return_ctx = parser.parse_return_ctx().unwrap();

        // Then
        assert_eq!(*return_ctx.return_tkn.lexeme_ref(), Lexeme::KwReturn);
    }

    #[test]
    fn parse_return_value_test() {
        // Given
        const SRC_TEXT: &str = "return 10\n";
        let mut parser = Parser::from_text(SRC_TEXT);

        // When
        let return_ctx = parser.parse_return_ctx().unwrap();

        // Then
        assert_eq!(*return_ctx.return_tkn.lexeme_ref(), Lexeme::KwReturn);
    }
}
