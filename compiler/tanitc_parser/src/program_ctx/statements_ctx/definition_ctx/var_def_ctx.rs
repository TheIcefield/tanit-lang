use tanitc_ast::program_ctx::statement_ctx::definition_ctx::var_def_ctx::{
    VarDefCtx, VarDefTypeCtx, VarDefValueCtx,
};
use tanitc_lexer::token::lexeme::Lexeme;

use crate::{ParseResult, Parser};

impl Parser {
    pub fn parse_var_def_ctx(&mut self) -> ParseResult<VarDefCtx> {
        Ok(VarDefCtx {
            attributes_ctx: Box::default(),
            var_tkn: self.consume_token(Lexeme::KwVar)?,
            mut_tkn: self.consume_token(Lexeme::KwMut).ok(),
            name_ctx: Box::new(self.parse_name_ctx()?),
            type_ctx: if self.is_next(Lexeme::Colon) {
                Some(self.parse_var_def_type_ctx()?)
            } else {
                None
            },
            value_ctx: if self.is_next(Lexeme::Assign) {
                Some(self.parse_var_def_value_ctx()?)
            } else {
                None
            },
        })
    }

    fn parse_var_def_type_ctx(&mut self) -> ParseResult<VarDefTypeCtx> {
        Ok(VarDefTypeCtx {
            colon_tkn: self.consume_token(Lexeme::Colon)?,
            type_ctx: Box::new(self.parse_type_ctx()?),
        })
    }

    fn parse_var_def_value_ctx(&mut self) -> ParseResult<VarDefValueCtx> {
        Ok(VarDefValueCtx {
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
        const SRC_TEXT: &str = "var var_name = 0\n";
        const VAR_NAME: &str = "var_name";

        let mut parser = Parser::from_text(SRC_TEXT);

        let var_def_ctx = parser.parse_var_def_ctx().unwrap();

        assert_eq!(var_def_ctx.name_ctx.to_string(), VAR_NAME);
        assert!(var_def_ctx.mut_tkn.is_none());
        assert!(var_def_ctx.type_ctx.is_none());
        assert!(var_def_ctx.value_ctx.is_some())
    }

    #[test]
    fn var_mut_def_test() {
        const SRC_TEXT: &str = "var mut var_name = 0\n";
        const VAR_NAME: &str = "var_name";

        let mut parser = Parser::from_text(SRC_TEXT);

        let var_def_ctx = parser.parse_var_def_ctx().unwrap();

        assert_eq!(var_def_ctx.name_ctx.to_string(), VAR_NAME);
        assert!(var_def_ctx.mut_tkn.is_some());
        assert!(var_def_ctx.type_ctx.is_none());
        assert!(var_def_ctx.value_ctx.is_some())
    }

    #[test]
    fn var_type_def_test() {
        const SRC_TEXT: &str = "var var_name: i32\n";
        const VAR_NAME: &str = "var_name";

        let mut parser = Parser::from_text(SRC_TEXT);

        let var_def_ctx = parser.parse_var_def_ctx().unwrap();

        assert_eq!(var_def_ctx.name_ctx.to_string(), VAR_NAME);
        assert!(var_def_ctx.mut_tkn.is_none());
        assert!(var_def_ctx.type_ctx.is_some());
        assert!(var_def_ctx.value_ctx.is_none())
    }
}
