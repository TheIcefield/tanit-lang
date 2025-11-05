use tanitc_ast::program_ctx::statement_ctx::definition_ctx::impl_def_ctx::{
    ImplDefBodyCtx, ImplDefCtx,
};
use tanitc_lexer::token::lexeme::Lexeme;

use crate::{ParseResult, Parser};

impl Parser {
    pub fn parse_impl_ctx(&mut self) -> ParseResult<ImplDefCtx> {
        Ok(ImplDefCtx {
            attributes_ctx: Box::default(),
            impl_tkn: self.consume_token(Lexeme::KwImpl)?,
            name_ctx: Box::new(self.parse_name_ctx()?),
            body_ctx: self.parse_impl_body_ctx()?,
        })
    }

    fn parse_impl_body_ctx(&mut self) -> ParseResult<ImplDefBodyCtx> {
        Ok(ImplDefBodyCtx {
            block_ctx: {
                let old_opt = self.does_ignore_nl();
                self.set_ignore_nl_option(true);

                let ctx = Box::new(self.parse_block_ctx()?);

                self.set_ignore_nl_option(old_opt);

                ctx
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::Parser;

    use tanitc_ast::program_ctx::statement_ctx::{definition_ctx::DefinitionCtx, StatementCtx};

    #[test]
    fn parse_empty_impl_test() {
        const SRC_TEXT: &str = "\nimpl MyStruct { }";

        let mut parser = Parser::from_text(SRC_TEXT);

        let impl_ctx = parser.parse_impl_ctx().unwrap();

        let errors = parser.messages_ref().errors_ref();
        if !errors.is_empty() {
            panic!("{errors:#?}");
        }

        assert!(impl_ctx
            .body_ctx
            .block_ctx
            .statements_ctx
            .statements
            .is_empty());
    }

    #[test]
    fn parse_impl_good_test() {
        const SRC_TEXT: &str = "\nimpl MyStruct\
                                \n{\
                                \n    func empty() {\
                                \n    }\
                                \n}";

        let mut parser = Parser::from_text(SRC_TEXT);

        let impl_ctx = parser.parse_impl_ctx().unwrap();
        let errors = parser.messages_ref().errors_ref();
        if !errors.is_empty() {
            panic!("{errors:#?}");
        }

        const METHODS_COUNT: usize = 2;

        assert_eq!(impl_ctx.name_ctx.to_string(), "MyStruct");
        assert_eq!(
            impl_ctx.body_ctx.block_ctx.statements_ctx.statements.len(),
            METHODS_COUNT
        );

        {
            const METHOD_INDEX: usize = 0;

            let (None, None) = &impl_ctx.body_ctx.block_ctx.statements_ctx.statements[METHOD_INDEX]
            else {
                panic!("Unexpected statement ctx");
            };
        }

        {
            const METHOD_INDEX: usize = 1;
            const METHOD_NAME: &str = "empty";

            let (Some(StatementCtx::Definition(DefinitionCtx::Func(func_def_ctx))), Some(_)) =
                &impl_ctx.body_ctx.block_ctx.statements_ctx.statements[METHOD_INDEX]
            else {
                panic!("Unexpected statement ctx");
            };

            assert_eq!(func_def_ctx.name_ctx.to_string(), METHOD_NAME);
            assert!(func_def_ctx.params_ctx.params_ctx.is_empty());
        }
    }
}
