use tanitc_ast::program_ctx::statement_ctx::definition_ctx::const_def_ctx::{
    ConstDefCtx, ConstDefTypeCtx, ConstDefValueCtx,
};
use tanitc_lexer::token::lexeme::Lexeme;

use crate::{ParseResult, Parser};

impl Parser {
    pub fn parse_const_def_ctx(&mut self) -> ParseResult<ConstDefCtx> {
        Ok(ConstDefCtx {
            attributes_ctx: Box::default(),
            const_tkn: self.consume_token(Lexeme::KwConst)?,
            name_ctx: Box::new(self.parse_name_ctx()?),
            type_ctx: self.parse_const_def_type_ctx()?,
            value_ctx: self.parse_const_def_value_ctx()?,
        })
    }

    fn parse_const_def_type_ctx(&mut self) -> ParseResult<ConstDefTypeCtx> {
        Ok(ConstDefTypeCtx {
            colon_tkn: self.consume_token(Lexeme::Colon)?,
            type_ctx: Box::new(self.parse_type_ctx()?),
        })
    }

    fn parse_const_def_value_ctx(&mut self) -> ParseResult<ConstDefValueCtx> {
        Ok(ConstDefValueCtx {
            equal_tkn: self.consume_token(Lexeme::Assign)?,
            value_ctx: Box::new(self.parse_expression_ctx()?),
        })
    }
}

#[cfg(test)]
mod tests {

    use crate::Parser;

    #[test]
    fn const_def_test() {
        const SRC_TEXT: &str = "const CONST_NAME: i32 = 0\n";
        const CONST_NAME: &str = "CONST_NAME";

        let mut parser = Parser::from_text(SRC_TEXT);

        let const_def_ctx = parser.parse_const_def_ctx().unwrap();

        assert_eq!(const_def_ctx.name_ctx.to_string(), CONST_NAME);
        assert!(const_def_ctx.type_ctx.type_ctx.is_named());
    }
}
