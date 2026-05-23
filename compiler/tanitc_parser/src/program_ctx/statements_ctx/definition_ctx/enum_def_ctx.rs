use tanitc_ast::program_ctx::statement_ctx::definition_ctx::enum_def_ctx::{
    EnumDefBodyCtx, EnumDefCtx, EnumDefUnitAssignCtx, EnumDefUnitCtx,
};
use tanitc_lexer::token::{lexeme::Lexeme, Token};
use tanitc_messages::Message;

use crate::{ParseResult, Parser};

impl Parser {
    pub fn parse_enum_def_ctx(&mut self) -> ParseResult<EnumDefCtx> {
        Ok(EnumDefCtx {
            attributes_ctx: Box::default(),
            enum_tkn: self.consume_token(Lexeme::KwEnum)?,
            name_ctx: Box::new(self.parse_name_ctx()?),
            body_ctx: self.parse_enum_def_body_ctx()?,
        })
    }

    fn parse_enum_def_body_ctx(&mut self) -> ParseResult<EnumDefBodyCtx> {
        Ok(EnumDefBodyCtx {
            lcb_tkn: self.consume_token(Lexeme::Lcb)?,
            units_ctx: {
                let old_opt = self.does_ignore_nl();
                self.set_ignore_nl_option(false);

                let ctx = self.parse_enum_def_body_ctx_internal()?;

                self.set_ignore_nl_option(old_opt);

                ctx
            },
            rcb_tkn: self.consume_token(Lexeme::Rcb)?,
        })
    }

    fn parse_enum_def_body_ctx_internal(
        &mut self,
    ) -> ParseResult<Vec<(Option<EnumDefUnitCtx>, Option<Token>)>> {
        let mut units = Vec::<(Option<EnumDefUnitCtx>, Option<Token>)>::new();

        while let Some(next) = self.peek_token() {
            let unit_ctx = match next.lexeme_ref() {
                Lexeme::Rcb => break,
                Lexeme::EndOfLine => None,
                Lexeme::Identifier(_) => Some(self.parse_enum_def_unit_ctx()?),
                Lexeme::Lcb => {
                    return Err(Message::new(
                        next.get_location(),
                        "Unexpected token: \"{{\" during parsing enum fields.\n\
                            Help: if you tried to declare struct-like field, place \"{{\" \
                            in the same line with name of the field.",
                    ));
                }
                _ => {
                    self.error(Message::unexpected_token(&next, &[]));
                    self.skip_until(&[Lexeme::EndOfLine]);

                    continue;
                }
            };

            let nl_tkn = self.consume_token(Lexeme::EndOfLine).ok();

            units.push((unit_ctx, nl_tkn));
        }

        Ok(units)
    }

    fn parse_enum_def_unit_ctx(&mut self) -> ParseResult<EnumDefUnitCtx> {
        Ok(EnumDefUnitCtx {
            name_ctx: Box::new(self.parse_name_ctx()?),
            assign_ctx: if self.is_next(Lexeme::Colon) {
                Some(EnumDefUnitAssignCtx {
                    colon_tkn: self.consume_token(Lexeme::Colon)?,
                    value_tkn: self.consume_integer()?,
                })
            } else {
                None
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use tanitc_lexer::token::lexeme::Lexeme;

    use crate::Parser;

    #[test]
    fn parse_enum_def_test() {
        const SRC_TEXT: &str = "\nenum MyEnum {\
                            \n    One: 1\
                            \n    Two\
                            \n    Max\
                            \n}";

        let mut parser = Parser::from_text(SRC_TEXT);

        let enum_def_ctx = parser.parse_enum_def_ctx().unwrap();

        assert_eq!(enum_def_ctx.name_ctx.to_string(), "MyEnum");
        assert_eq!(*enum_def_ctx.body_ctx.lcb_tkn.lexeme_ref(), Lexeme::Lcb);
        assert_eq!(*enum_def_ctx.body_ctx.rcb_tkn.lexeme_ref(), Lexeme::Rcb);

        // println!("MyEnum: {:#?}", enum_def_ctx.body_ctx.units_ctx);
        assert_eq!(enum_def_ctx.body_ctx.units_ctx.len(), 4);

        {
            let (None, Some(nl_tkn)) = &enum_def_ctx.body_ctx.units_ctx[0] else {
                panic!("Unexpected field ctx");
            };

            assert_eq!(*nl_tkn.lexeme_ref(), Lexeme::EndOfLine);
        }

        {
            let (Some(unit_ctx), Some(nl_tkn)) = &enum_def_ctx.body_ctx.units_ctx[1] else {
                panic!("Unexpected field ctx");
            };

            assert_eq!(unit_ctx.name_ctx.to_string(), "One");
            assert!(unit_ctx.assign_ctx.is_some());
            if let Some(assign_ctx) = unit_ctx.assign_ctx.as_ref() {
                assert_eq!(*assign_ctx.colon_tkn.lexeme_ref(), Lexeme::Colon);
                assert_eq!(
                    *assign_ctx.value_tkn.lexeme_ref(),
                    Lexeme::Integer(1.to_string())
                );
            }

            assert_eq!(*nl_tkn.lexeme_ref(), Lexeme::EndOfLine);
        }

        {
            let (Some(unit_ctx), Some(nl_tkn)) = &enum_def_ctx.body_ctx.units_ctx[2] else {
                panic!("Unexpected field ctx");
            };

            assert_eq!(unit_ctx.name_ctx.to_string(), "Two");
            assert!(unit_ctx.assign_ctx.is_none());
            assert_eq!(*nl_tkn.lexeme_ref(), Lexeme::EndOfLine);
        }

        {
            let (Some(unit_ctx), Some(nl_tkn)) = &enum_def_ctx.body_ctx.units_ctx[3] else {
                panic!("Unexpected field ctx");
            };

            assert_eq!(unit_ctx.name_ctx.to_string(), "Max");
            assert!(unit_ctx.assign_ctx.is_none());
            assert_eq!(*nl_tkn.lexeme_ref(), Lexeme::EndOfLine);
        }
    }

    #[test]
    fn parse_empty_enum_def_test() {
        const SRC_TEXT: &str = "\nenum EmptyEnum { }";

        let mut parser = Parser::from_text(SRC_TEXT);

        let enum_def_ctx = parser.parse_enum_def_ctx().unwrap();

        assert_eq!(enum_def_ctx.name_ctx.to_string(), "EmptyEnum");
        assert_eq!(*enum_def_ctx.body_ctx.lcb_tkn.lexeme_ref(), Lexeme::Lcb);
        assert_eq!(*enum_def_ctx.body_ctx.rcb_tkn.lexeme_ref(), Lexeme::Rcb);
        assert!(enum_def_ctx.body_ctx.units_ctx.is_empty());
    }

    #[test]
    fn parse_enum_with_one_field_def_test() {
        const SRC_TEXT: &str = "\nenum MyEnum { MinsInHour: 60\n }";

        let mut parser = Parser::from_text(SRC_TEXT);

        let enum_def_ctx = parser.parse_enum_def_ctx().unwrap();

        assert_eq!(enum_def_ctx.name_ctx.to_string(), "MyEnum");
        assert_eq!(*enum_def_ctx.body_ctx.lcb_tkn.lexeme_ref(), Lexeme::Lcb);
        assert_eq!(*enum_def_ctx.body_ctx.rcb_tkn.lexeme_ref(), Lexeme::Rcb);
        assert_eq!(enum_def_ctx.body_ctx.units_ctx.len(), 1);

        {
            let (Some(unit_ctx), Some(nl_tkn)) = &enum_def_ctx.body_ctx.units_ctx[0] else {
                panic!("Unexpected field ctx");
            };

            assert_eq!(unit_ctx.name_ctx.to_string(), "MinsInHour");
            let Some(assign_ctx) = unit_ctx.assign_ctx.as_ref() else {
                panic!("Expected assign ctx");
            };
            assert_eq!(*assign_ctx.colon_tkn.lexeme_ref(), Lexeme::Colon);
            assert_eq!(
                *assign_ctx.value_tkn.lexeme_ref(),
                Lexeme::Integer(60.to_string())
            );
            assert_eq!(Lexeme::EndOfLine, *nl_tkn.lexeme_ref());
        }
    }
}
