use tanitc_ast::program_ctx::statement_ctx::definition_ctx::struct_def_ctx::{
    StructDefBodyCtx, StructDefCtx, StructDefFieldCtx,
};
use tanitc_lexer::token::{lexeme::Lexeme, Token};
use tanitc_messages::Message;

use crate::{ParseResult, Parser};

impl Parser {
    pub fn parse_struct_def_ctx(&mut self) -> ParseResult<StructDefCtx> {
        Ok(StructDefCtx {
            attributes_ctx: Box::default(),
            struct_tkn: self.consume_token(Lexeme::KwStruct)?,
            name_ctx: Box::new(self.parse_name_ctx()?),
            body_ctx: self.parse_struct_def_body_ctx()?,
        })
    }

    pub(crate) fn parse_struct_def_body_ctx(&mut self) -> ParseResult<StructDefBodyCtx> {
        Ok(StructDefBodyCtx {
            lcb_tkn: self.consume_token(Lexeme::Lcb)?,
            fields_ctx: {
                let old_opt = self.does_ignore_nl();

                self.set_ignore_nl_option(false);
                let ctx = self.parse_struct_fields_internal()?;
                self.set_ignore_nl_option(old_opt);

                ctx
            },
            rcb_tkn: self.consume_token(Lexeme::Rcb)?,
        })
    }

    fn parse_struct_fields_internal(
        &mut self,
    ) -> ParseResult<Vec<(Option<StructDefFieldCtx>, Option<Token>)>> {
        let mut fields = Vec::<(Option<StructDefFieldCtx>, Option<Token>)>::new();

        while let Some(next) = self.peek_token() {
            let field_ctx = match next.lexeme_ref() {
                Lexeme::Rcb => break,

                Lexeme::EndOfLine => None,

                Lexeme::Identifier(_) => Some(StructDefFieldCtx {
                    pub_tkn: self.consume_token(Lexeme::KwPub).ok(),
                    name_ctx: Box::new(self.parse_name_ctx()?),
                    colon_tkn: self.consume_token(Lexeme::Colon)?,
                    type_ctx: Box::new(self.parse_type_ctx()?),
                }),

                _ => {
                    self.error(Message::unexpected_token(&next, &[]));
                    self.skip_until(&[Lexeme::EndOfLine]);

                    continue;
                }
            };

            let nl_tkn = self.consume_token(Lexeme::EndOfLine).ok();

            fields.push((field_ctx, nl_tkn));
        }

        Ok(fields)
    }
}

#[cfg(test)]
mod tests {
    use tanitc_ast::program_ctx::type_ctx::TypeCtx;
    use tanitc_lexer::token::lexeme::Lexeme;

    use crate::Parser;

    #[test]
    fn parse_struct_def() {
        const SRC_TEXT: &str = r#"
            struct Foo {
                x: i32
                y: f64
            }
        "#;

        let mut parser = Parser::from_text(SRC_TEXT);
        let struct_def_ctx = parser.parse_struct_def_ctx().unwrap();

        assert_eq!(struct_def_ctx.name_ctx.to_string(), "Foo");
        assert_eq!(*struct_def_ctx.body_ctx.lcb_tkn.lexeme_ref(), Lexeme::Lcb);
        assert_eq!(*struct_def_ctx.body_ctx.rcb_tkn.lexeme_ref(), Lexeme::Rcb);
        assert_eq!(struct_def_ctx.body_ctx.fields_ctx.len(), 3);

        {
            let (None, Some(nl_tkn)) = &struct_def_ctx.body_ctx.fields_ctx[0] else {
                panic!("Unexpected field context");
            };
            assert_eq!(*nl_tkn.lexeme_ref(), Lexeme::EndOfLine);
        }

        {
            let (Some(field_ctx), Some(nl_tkn)) = &struct_def_ctx.body_ctx.fields_ctx[1] else {
                panic!("Unexpected field context");
            };

            assert_eq!(field_ctx.name_ctx.to_string(), "x");

            let TypeCtx::Named(field_type_ctx) = field_ctx.type_ctx.as_ref() else {
                panic!("Unexpected type context: {}", field_ctx.type_ctx.kind_str());
            };
            assert_eq!(field_type_ctx.name_ctx.to_string(), "i32");

            assert_eq!(*nl_tkn.lexeme_ref(), Lexeme::EndOfLine);
        }

        {
            let (Some(field_ctx), Some(nl_tkn)) = &struct_def_ctx.body_ctx.fields_ctx[2] else {
                panic!("Unexpected field context");
            };

            assert_eq!(field_ctx.name_ctx.to_string(), "y");

            let TypeCtx::Named(field_type_ctx) = field_ctx.type_ctx.as_ref() else {
                panic!("Unexpected type context: {}", field_ctx.type_ctx.kind_str());
            };
            assert_eq!(field_type_ctx.name_ctx.to_string(), "f64");

            assert_eq!(*nl_tkn.lexeme_ref(), Lexeme::EndOfLine);
        }
    }

    #[test]
    fn parse_empty_struct_def() {
        const SRC_TEXT: &str = r#"
            struct Foo { }
        "#;

        let mut parser = Parser::from_text(SRC_TEXT);
        let struct_def_ctx = parser.parse_struct_def_ctx().unwrap();

        assert_eq!(struct_def_ctx.name_ctx.to_string(), "Foo");
        assert_eq!(*struct_def_ctx.body_ctx.lcb_tkn.lexeme_ref(), Lexeme::Lcb);
        assert_eq!(*struct_def_ctx.body_ctx.rcb_tkn.lexeme_ref(), Lexeme::Rcb);
        assert!(struct_def_ctx.body_ctx.fields_ctx.is_empty());
    }
}
