use tanitc_ast::program_ctx::statement_ctx::expression_ctx::{
    literal_ctx::array_literal_ctx::ArrayLiteralCtx, ExpressionCtx,
};
use tanitc_lexer::token::{lexeme::Lexeme, Token};

use crate::{ParseResult, Parser};

impl Parser {
    pub(crate) fn parse_array_literal_ctx(&mut self) -> ParseResult<ArrayLiteralCtx> {
        Ok(ArrayLiteralCtx {
            lsb_tkn: self.consume_token(Lexeme::Lsb)?,
            elements: self.parse_array_elements()?,
            rsb_tkn: self.consume_token(Lexeme::Rsb)?,
        })
    }

    fn parse_array_elements(&mut self) -> ParseResult<Vec<(Option<ExpressionCtx>, Option<Token>)>> {
        let mut elements = Vec::<(Option<ExpressionCtx>, Option<Token>)>::new();

        let old_opt = self.does_ignore_nl();
        self.set_ignore_nl_option(true);
        while let Some(next) = self.peek_token() {
            if *next.lexeme_ref() == Lexeme::Rcb {
                break;
            }

            let expression_ctx = self.parse_expression_ctx();

            if let Err(err) = &expression_ctx {
                self.error(err.clone());
                self.skip_until(&[Lexeme::Comma]);
            }

            elements.push((expression_ctx.ok(), self.consume_token(Lexeme::Comma).ok()));
        }
        self.set_ignore_nl_option(old_opt);

        Ok(elements)
    }
}
