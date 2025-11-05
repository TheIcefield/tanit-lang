use tanitc_ast::program_ctx::{
    name_ctx::NameCtx,
    statement_ctx::definition_ctx::variant_def_ctx::{
        VariantDefBodyCtx, VariantDefCtx, VariantDefEnumFieldCtx, VariantDefFieldCtx,
        VariantDefStructFieldCtx, VariantDefTupleFieldCtx,
    },
};
use tanitc_lexer::token::{lexeme::Lexeme, Token};
use tanitc_messages::Message;

use crate::{ParseResult, Parser};

impl Parser {
    pub fn parse_variant_def_ctx(&mut self) -> ParseResult<VariantDefCtx> {
        Ok(VariantDefCtx {
            attributes_ctx: Box::default(),
            variant_tkn: self.consume_token(Lexeme::KwVariant)?,
            name_ctx: Box::new(self.parse_name_ctx()?),
            body_ctx: self.parse_variant_def_body_ctx()?,
        })
    }

    fn parse_variant_def_body_ctx(&mut self) -> ParseResult<VariantDefBodyCtx> {
        Ok(VariantDefBodyCtx {
            lcb_tkn: self.consume_token(Lexeme::Lcb)?,
            fields_ctx: {
                let old_opt = self.does_ignore_nl();
                self.set_ignore_nl_option(false);

                let ctx = self.parse_variant_def_body_ctx_internal()?;

                self.set_ignore_nl_option(old_opt);
                ctx
            },
            rcb_tkn: self.consume_token(Lexeme::Rcb)?,
        })
    }

    fn parse_variant_def_body_ctx_internal(
        &mut self,
    ) -> ParseResult<Vec<(Option<VariantDefFieldCtx>, Option<Token>)>> {
        let mut fields = Vec::<(Option<VariantDefFieldCtx>, Option<Token>)>::new();

        loop {
            let Some(next) = self.peek_token() else {
                break;
            };

            let field_ctx = match next.lexeme_ref() {
                Lexeme::Rcb => break,
                Lexeme::EndOfLine => None,
                Lexeme::Identifier(_) => Some(self.parse_variant_field()?),
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

    fn parse_variant_field(&mut self) -> ParseResult<VariantDefFieldCtx> {
        let old_opt = self.does_ignore_nl();
        self.set_ignore_nl_option(false);

        let ctx = self.parse_variant_field_internal()?;

        self.set_ignore_nl_option(old_opt);

        Ok(ctx)
    }

    fn parse_variant_field_internal(&mut self) -> ParseResult<VariantDefFieldCtx> {
        let name_ctx = Box::new(self.parse_name_ctx()?);

        let next = self.peek_token().ok_or(Message::reached_eof())?;
        match next.lexeme_ref() {
            Lexeme::EndOfLine => self
                .parse_enum_variant_field(name_ctx)
                .map(VariantDefFieldCtx::Enum),

            Lexeme::LParen => self
                .parse_tuple_variant_field(name_ctx)
                .map(VariantDefFieldCtx::Tuple),

            Lexeme::Lcb => self
                .parse_struct_variant_field(name_ctx)
                .map(VariantDefFieldCtx::Struct),

            _ => Err(Message::unexpected_token(&next, &[])),
        }
    }

    fn parse_enum_variant_field(
        &mut self,
        name_ctx: Box<NameCtx>,
    ) -> ParseResult<VariantDefEnumFieldCtx> {
        Ok(VariantDefEnumFieldCtx { name_ctx })
    }

    fn parse_tuple_variant_field(
        &mut self,
        name_ctx: Box<NameCtx>,
    ) -> ParseResult<VariantDefTupleFieldCtx> {
        Ok(VariantDefTupleFieldCtx {
            name_ctx,
            tuple_type_ctx: Box::new(self.parse_tuple_type_ctx()?),
        })
    }

    fn parse_struct_variant_field(
        &mut self,
        name_ctx: Box<NameCtx>,
    ) -> ParseResult<VariantDefStructFieldCtx> {
        Ok(VariantDefStructFieldCtx {
            name_ctx,
            struct_body_ctx: Box::new(self.parse_struct_def_body_ctx()?),
        })
    }
}

#[cfg(test)]
mod tests {
    use tanitc_ast::program_ctx::{
        statement_ctx::definition_ctx::{
            struct_def_ctx::StructDefFieldCtx, variant_def_ctx::VariantDefFieldCtx,
        },
        type_ctx::TypeCtx,
    };
    use tanitc_lexer::token::lexeme::Lexeme;

    use crate::Parser;

    #[test]
    fn parse_variant_def() {
        const SRC_TEXT: &str = r#"
            variant Foo {
                First
                Second { a: i32 }
                Third (i32, f32)
            }
        "#;

        let mut parser = Parser::from_text(SRC_TEXT);
        let variant_def_ctx = parser.parse_variant_def_ctx().unwrap();

        assert_eq!(variant_def_ctx.name_ctx.to_string(), "Foo");
        assert_eq!(*variant_def_ctx.body_ctx.lcb_tkn.lexeme_ref(), Lexeme::Lcb);
        assert_eq!(*variant_def_ctx.body_ctx.rcb_tkn.lexeme_ref(), Lexeme::Rcb);
        assert_eq!(variant_def_ctx.body_ctx.fields_ctx.len(), 4);

        {
            let (None, Some(nl_tkn)) = &variant_def_ctx.body_ctx.fields_ctx[0] else {
                panic!("Unexpected field context");
            };
            assert_eq!(*nl_tkn.lexeme_ref(), Lexeme::EndOfLine);
        }

        {
            let (Some(VariantDefFieldCtx::Enum(field_ctx)), Some(nl_tkn)) =
                &variant_def_ctx.body_ctx.fields_ctx[1]
            else {
                panic!("Unexpected variant field context");
            };

            assert_eq!(field_ctx.name_ctx.to_string(), "First");
            assert_eq!(*nl_tkn.lexeme_ref(), Lexeme::EndOfLine);
        }

        {
            let (Some(VariantDefFieldCtx::Struct(field_ctx)), Some(nl_tkn)) =
                &variant_def_ctx.body_ctx.fields_ctx[2]
            else {
                panic!("Unexpected variant field context");
            };

            assert_eq!(field_ctx.name_ctx.to_string(), "Second");
            assert_eq!(*field_ctx.struct_body_ctx.lcb_tkn.lexeme_ref(), Lexeme::Lcb);
            assert_eq!(*field_ctx.struct_body_ctx.rcb_tkn.lexeme_ref(), Lexeme::Rcb);

            let fields = &field_ctx.struct_body_ctx.fields_ctx;
            assert_eq!(fields.len(), 1);

            let (
                Some(StructDefFieldCtx {
                    pub_tkn,
                    name_ctx,
                    colon_tkn,
                    type_ctx,
                }),
                None,
            ) = &fields[0]
            else {
                panic!("Unexpected field context");
            };
            assert!(pub_tkn.is_none());
            assert_eq!(name_ctx.to_string(), "a");
            assert_eq!(*colon_tkn.lexeme_ref(), Lexeme::Colon);

            let TypeCtx::Named(field_type_ctx) = type_ctx.as_ref() else {
                panic!("Unexpected type context: {}", type_ctx.kind_str());
            };
            assert_eq!(field_type_ctx.name_ctx.to_string(), "i32");

            assert_eq!(*nl_tkn.lexeme_ref(), Lexeme::EndOfLine);
        }

        {
            let (Some(VariantDefFieldCtx::Tuple(field_ctx)), Some(nl_tkn)) =
                &variant_def_ctx.body_ctx.fields_ctx[3]
            else {
                panic!("Unexpected variant field context");
            };

            assert_eq!(field_ctx.name_ctx.to_string(), "Third");
            assert_eq!(
                *field_ctx.tuple_type_ctx.lparen_tkn.lexeme_ref(),
                Lexeme::LParen
            );
            assert_eq!(
                *field_ctx.tuple_type_ctx.rparen_tkn.lexeme_ref(),
                Lexeme::RParen
            );

            let units = &field_ctx.tuple_type_ctx.units_ctx;
            assert_eq!(units.len(), 2);

            {
                let unit_ctx = &units[0];
                assert!(unit_ctx
                    .comma_tkn
                    .as_ref()
                    .is_some_and(|tkn| Lexeme::Comma == *tkn.lexeme_ref()));

                let TypeCtx::Named(field_type_ctx) = unit_ctx.type_ctx.as_ref() else {
                    panic!("Unexpected type context: {}", unit_ctx.type_ctx.kind_str());
                };
                assert_eq!(field_type_ctx.name_ctx.to_string(), "i32");
            }

            {
                let unit_ctx = &units[1];
                assert!(unit_ctx.comma_tkn.is_none());

                let TypeCtx::Named(field_type_ctx) = unit_ctx.type_ctx.as_ref() else {
                    panic!("Unexpected type context: {}", unit_ctx.type_ctx.kind_str());
                };
                assert_eq!(field_type_ctx.name_ctx.to_string(), "f32");
            }

            assert_eq!(*nl_tkn.lexeme_ref(), Lexeme::EndOfLine);
        }
    }

    #[test]
    fn parse_empty_variant_def() {
        const SRC_TEXT: &str = r#"
            variant Foo { }
        "#;

        let mut parser = Parser::from_text(SRC_TEXT);
        let variant_def_ctx = parser.parse_variant_def_ctx().unwrap();

        assert_eq!(variant_def_ctx.name_ctx.to_string(), "Foo");
        assert_eq!(*variant_def_ctx.body_ctx.lcb_tkn.lexeme_ref(), Lexeme::Lcb);
        assert_eq!(*variant_def_ctx.body_ctx.rcb_tkn.lexeme_ref(), Lexeme::Rcb);
        assert!(variant_def_ctx.body_ctx.fields_ctx.is_empty());
    }
}
