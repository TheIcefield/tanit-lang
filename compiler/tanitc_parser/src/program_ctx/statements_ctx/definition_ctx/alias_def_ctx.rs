use tanitc_ast::program_ctx::statement_ctx::definition_ctx::alias_def_ctx::AliasDefCtx;
use tanitc_lexer::token::lexeme::Lexeme;

use crate::{ParseResult, Parser};

impl Parser {
    pub fn parse_alias_def_ctx(&mut self) -> ParseResult<AliasDefCtx> {
        Ok(AliasDefCtx {
            attributes_ctx: Box::default(),
            alias_tkn: self.consume_token(Lexeme::KwAlias)?,
            name_ctx: Box::new(self.parse_name_ctx()?),
            assign_tkn: self.consume_token(Lexeme::Assign)?,
            type_ctx: Box::new(self.parse_type_ctx()?),
        })
    }
}

#[cfg(test)]
mod tests {
    use tanitc_lexer::token::lexeme::Lexeme;

    use crate::Parser;

    #[test]
    fn parse_alias_def_test() {
        const SRC_TEXT: &str = "alias MyAlias = f32";

        let mut parser = Parser::from_text(SRC_TEXT);
        let alias_def_ctx = parser.parse_alias_def_ctx().unwrap();

        assert_eq!(*alias_def_ctx.alias_tkn.lexeme_ref(), Lexeme::KwAlias);
        assert_eq!(alias_def_ctx.name_ctx.identifier().to_string(), "MyAlias");
        assert_eq!(*alias_def_ctx.assign_tkn.lexeme_ref(), Lexeme::Assign);
        assert!(alias_def_ctx.type_ctx.is_named());
    }
}
