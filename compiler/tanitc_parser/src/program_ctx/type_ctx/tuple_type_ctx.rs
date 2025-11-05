use tanitc_ast::program_ctx::type_ctx::tuple_type_ctx::{TupleTypeCtx, TupleTypeUnitCtx};
use tanitc_lexer::token::lexeme::Lexeme;

use crate::{ParseResult, Parser};

impl Parser {
    pub(crate) fn parse_tuple_type_ctx(&mut self) -> ParseResult<TupleTypeCtx> {
        Ok(TupleTypeCtx {
            lparen_tkn: self.consume_token(Lexeme::LParen)?,
            units_ctx: self.parse_tuple_units_ctx()?,
            rparen_tkn: self.consume_token(Lexeme::RParen)?,
        })
    }

    fn parse_tuple_units_ctx(&mut self) -> ParseResult<Vec<TupleTypeUnitCtx>> {
        let mut units = Vec::<TupleTypeUnitCtx>::new();

        loop {
            if self.is_next(Lexeme::RParen) {
                break;
            }

            units.push(self.parse_tuple_unit_ctx()?);
        }

        Ok(units)
    }

    fn parse_tuple_unit_ctx(&mut self) -> ParseResult<TupleTypeUnitCtx> {
        Ok(TupleTypeUnitCtx {
            type_ctx: Box::new(self.parse_type_ctx()?),
            comma_tkn: self.consume_token(Lexeme::Comma).ok(),
        })
    }
}

#[cfg(test)]
mod tests {
    use tanitc_lexer::token::lexeme::Lexeme;

    use crate::Parser;

    #[test]
    fn parse_common_tuple_type_ctx_test() {
        const SRC_TEXT: &str = "(i32, i32)";

        let mut parser = Parser::from_text(SRC_TEXT);
        let type_ctx = parser.parse_tuple_type_ctx().unwrap();

        assert_eq!(*type_ctx.lparen_tkn.lexeme_ref(), Lexeme::LParen);
        assert!(type_ctx.units_ctx[0].type_ctx.is_named());
        assert!(type_ctx.units_ctx[1].type_ctx.is_named());
        assert_eq!(*type_ctx.rparen_tkn.lexeme_ref(), Lexeme::RParen);
    }

    #[test]
    fn parse_empty_tuple_type_ctx_test() {
        const SRC_TEXT: &str = "()";

        let mut parser = Parser::from_text(SRC_TEXT);
        let type_ctx = parser.parse_tuple_type_ctx().unwrap();

        assert_eq!(*type_ctx.lparen_tkn.lexeme_ref(), Lexeme::LParen);
        assert!(type_ctx.units_ctx.is_empty());
        assert_eq!(*type_ctx.rparen_tkn.lexeme_ref(), Lexeme::RParen);
    }

    #[test]
    fn parse_multiline_tuple_type_ctx_test_1() {
        const SRC_TEXT: &str = "(i32,\
                               \ni32)";

        let mut parser = Parser::from_text(SRC_TEXT);
        let type_ctx = parser.parse_tuple_type_ctx().unwrap();

        assert_eq!(*type_ctx.lparen_tkn.lexeme_ref(), Lexeme::LParen);
        assert_eq!(type_ctx.units_ctx.len(), 2);
        assert!(type_ctx.units_ctx[0].type_ctx.is_named());
        assert!(type_ctx.units_ctx[1].type_ctx.is_named());
        assert_eq!(*type_ctx.rparen_tkn.lexeme_ref(), Lexeme::RParen);
    }

    #[test]
    fn parse_multiline_tuple_type_ctx_test_2() {
        const SRC_TEXT: &str = "(\
                               \ni32,\
                               \n\
                               \ni32\
                               \n)";

        let mut parser = Parser::from_text(SRC_TEXT);
        let type_ctx = parser.parse_tuple_type_ctx().unwrap();

        assert_eq!(*type_ctx.lparen_tkn.lexeme_ref(), Lexeme::LParen);
        assert_eq!(type_ctx.units_ctx.len(), 2);
        assert!(type_ctx.units_ctx[0].type_ctx.is_named());
        assert!(type_ctx.units_ctx[1].type_ctx.is_named());
        assert_eq!(*type_ctx.rparen_tkn.lexeme_ref(), Lexeme::RParen);
    }

    #[test]
    fn parse_multiline_empty_tuple_type_ctx_test() {
        const SRC_TEXT: &str = "(\
                              \n\
                              \n)";

        let mut parser = Parser::from_text(SRC_TEXT);
        let type_ctx = parser.parse_tuple_type_ctx().unwrap();

        assert_eq!(*type_ctx.lparen_tkn.lexeme_ref(), Lexeme::LParen);
        assert!(type_ctx.units_ctx.is_empty());
        assert_eq!(*type_ctx.rparen_tkn.lexeme_ref(), Lexeme::RParen);
    }
}
