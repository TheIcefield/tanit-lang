use tanitc_ast::program_ctx::statement_ctx::definition_ctx::static_def_ctx::{
    StaticDefCtx, StaticDefTypeCtx, StaticDefValueCtx,
};
use tanitc_lexer::token::lexeme::Lexeme;

use crate::{ParseResult, Parser};

impl Parser {
    pub fn parse_static_def_ctx(&mut self) -> ParseResult<StaticDefCtx> {
        Ok(StaticDefCtx {
            attributes_ctx: Box::default(),
            static_tkn: self.consume_token(Lexeme::KwStatic)?,
            mut_tkn: self.consume_token(Lexeme::KwMut).ok(),
            name_ctx: Box::new(self.parse_name_ctx()?),
            type_ctx: self.parse_static_def_type_ctx()?,
            value_ctx: self.parse_static_def_value_ctx()?,
        })
    }

    fn parse_static_def_type_ctx(&mut self) -> ParseResult<StaticDefTypeCtx> {
        Ok(StaticDefTypeCtx {
            colon_tkn: self.consume_token(Lexeme::Colon)?,
            type_ctx: Box::new(self.parse_type_ctx()?),
        })
    }

    fn parse_static_def_value_ctx(&mut self) -> ParseResult<StaticDefValueCtx> {
        Ok(StaticDefValueCtx {
            equal_tkn: self.consume_token(Lexeme::Assign)?,
            value_ctx: Box::new(self.parse_expression_ctx()?),
        })
    }
}

#[cfg(test)]
mod tests {

    use crate::Parser;

    #[test]
    fn var_def_test() {
        const SRC_TEXT: &str = "static STATIC_NAME: i32 = 0\n";
        const STATIC_NAME: &str = "STATIC_NAME";

        let mut parser = Parser::from_text(SRC_TEXT);

        let static_def_ctx = parser.parse_static_def_ctx().unwrap();

        assert_eq!(static_def_ctx.name_ctx.to_string(), STATIC_NAME);
        assert!(static_def_ctx.mut_tkn.is_none());
        assert!(static_def_ctx.type_ctx.type_ctx.is_named());
        assert!(static_def_ctx.value_ctx.value_ctx.is_literal());
    }

    #[test]
    fn static_mut_def_test() {
        const SRC_TEXT: &str = "static mut STATIC_NAME: i32 = 0\n";
        const STATIC_NAME: &str = "STATIC_NAME";

        let mut parser = Parser::from_text(SRC_TEXT);

        let static_def_ctx = parser.parse_static_def_ctx().unwrap();

        assert_eq!(static_def_ctx.name_ctx.to_string(), STATIC_NAME);
        assert!(static_def_ctx.mut_tkn.is_some());
        assert!(static_def_ctx.type_ctx.type_ctx.is_named());
        assert!(static_def_ctx.value_ctx.value_ctx.is_literal());
    }
}
