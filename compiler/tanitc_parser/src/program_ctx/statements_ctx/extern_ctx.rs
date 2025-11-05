use tanitc_ast::program_ctx::statement_ctx::definition_ctx::extern_ctx::ExternCtx;
use tanitc_lexer::token::lexeme::Lexeme;

use crate::{ParseResult, Parser};

impl Parser {
    pub fn parse_extern_ctx(&mut self) -> ParseResult<ExternCtx> {
        Ok(ExternCtx {
            attributes_ctx: Box::default(),
            extern_tkn: self.consume_token(Lexeme::KwExtern)?,
            abi_tkn: self.consume_text()?,
            body_ctx: Box::new(self.parse_block_ctx()?),
        })
    }
}

#[cfg(test)]
mod tests {
    use tanitc_ast::program_ctx::{
        statement_ctx::{definition_ctx::DefinitionCtx, StatementCtx},
        type_ctx::{tuple_type_ctx::TupleTypeCtx, TypeCtx},
    };
    use tanitc_lexer::token::lexeme::Lexeme;

    use crate::Parser;

    #[test]
    fn parse_extern_ctx_test() {
        const SRC_TEXT: &str = "extern \"C\" {\
                              \n    unsafe func hello(): ()\
                              \n}";

        let mut parser = Parser::from_text(SRC_TEXT);
        let extern_ctx = parser.parse_extern_ctx().unwrap();

        assert_eq!(*extern_ctx.extern_tkn.lexeme_ref(), Lexeme::KwExtern);
        assert_eq!(
            *extern_ctx.abi_tkn.lexeme_ref(),
            Lexeme::Text("\"C\"".to_string())
        );
        assert_eq!(*extern_ctx.body_ctx.lcb_tkn.lexeme_ref(), Lexeme::Lcb);
        assert_eq!(*extern_ctx.body_ctx.rcb_tkn.lexeme_ref(), Lexeme::Rcb);

        assert_eq!(extern_ctx.body_ctx.statements_ctx.statements.len(), 2);

        let (None, None) = &extern_ctx.body_ctx.statements_ctx.statements[0] else {
            panic!("Unexpected statement");
        };

        let (Some(StatementCtx::Definition(DefinitionCtx::Func(func_def_ctx))), Some(nl_tkn)) =
            &extern_ctx.body_ctx.statements_ctx.statements[1]
        else {
            panic!("Unexpected statement");
        };

        assert_eq!(*nl_tkn.lexeme_ref(), Lexeme::EndOfLine);
        assert_eq!(func_def_ctx.name_ctx.to_string(), "hello");
        assert!(func_def_ctx.params_ctx.params_ctx.is_empty());
        assert!(func_def_ctx.body_ctx.is_none());

        let Some(return_type_ctx) = &func_def_ctx.return_type_ctx else {
            unreachable!()
        };
        assert_eq!(*return_type_ctx.colon_tkn.lexeme_ref(), Lexeme::Colon);

        let TypeCtx::Tuple(TupleTypeCtx {
            lparen_tkn,
            units_ctx,
            rparen_tkn,
        }) = return_type_ctx.type_ctx.as_ref()
        else {
            panic!("Expected NamedTypeCtx")
        };
        assert_eq!(*lparen_tkn.lexeme_ref(), Lexeme::LParen);
        assert_eq!(*rparen_tkn.lexeme_ref(), Lexeme::RParen);
        assert!(units_ctx.is_empty());
    }
}
