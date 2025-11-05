use tanitc_ast::program_ctx::statement_ctx::block_ctx::BlockCtx;
use tanitc_lexer::token::lexeme::Lexeme;

use crate::{ParseResult, Parser};

impl Parser {
    pub fn parse_block_ctx(&mut self) -> ParseResult<BlockCtx> {
        Ok(BlockCtx {
            attributes_ctx: Box::default(),
            lcb_tkn: self.consume_token(Lexeme::Lcb)?,
            statements_ctx: {
                let old_opt = self.does_ignore_nl();
                self.set_ignore_nl_option(false);

                let items = self.parse_statements_ctx()?;

                self.set_ignore_nl_option(old_opt);

                items
            },
            rcb_tkn: self.consume_token(Lexeme::Rcb)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use tanitc_ast::program_ctx::statement_ctx::StatementCtx;
    use tanitc_lexer::token::lexeme::Lexeme;

    use crate::Parser;

    #[test]
    fn parse_block_ctx_test() {
        const SRC_TEXT: &str = "{\
                              \n    unsafe { }\
                              \n{}\
                              \n}";

        let mut parser = Parser::from_text(SRC_TEXT);
        let block_ctx = parser.parse_block_ctx().unwrap();

        assert_eq!(*block_ctx.lcb_tkn.lexeme_ref(), Lexeme::Lcb);
        assert_eq!(*block_ctx.rcb_tkn.lexeme_ref(), Lexeme::Rcb);

        let items = &block_ctx.statements_ctx;
        assert_eq!(items.statements.len(), 3);

        {
            let (None, None) = &items.statements[0] else {
                panic!("Unexpected statement ctx");
            };
        }

        {
            let (Some(StatementCtx::Block(block_ctx)), Some(nl_tkn)) = &items.statements[1] else {
                panic!("Unexpected statement ctx");
            };

            assert_eq!(block_ctx.attributes_ctx.safe_tkn, None);
            assert_eq!(block_ctx.attributes_ctx.pub_tkn, None);
            assert!(block_ctx
                .attributes_ctx
                .unsafe_tkn
                .as_ref()
                .is_some_and(|tkn| *tkn.lexeme_ref() == Lexeme::KwUnsafe));
            assert!(block_ctx.statements_ctx.statements.is_empty());
            assert_eq!(*nl_tkn.lexeme_ref(), Lexeme::EndOfLine);
        }

        {
            let (Some(StatementCtx::Block(block_ctx)), Some(nl_tkn)) = &items.statements[2] else {
                panic!("Unexpected statement ctx");
            };

            assert_eq!(block_ctx.attributes_ctx.safe_tkn, None);
            assert_eq!(block_ctx.attributes_ctx.unsafe_tkn, None);
            assert_eq!(block_ctx.attributes_ctx.pub_tkn, None);
            assert!(block_ctx.statements_ctx.statements.is_empty());
            assert_eq!(*nl_tkn.lexeme_ref(), Lexeme::EndOfLine);
        }
    }
}
