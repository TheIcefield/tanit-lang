use tanitc_ast::program_ctx::{
    name_ctx::NameCtx,
    statement_ctx::definition_ctx::func_def_ctx::{
        FuncDefCommonParamCtx, FuncDefCtx, FuncDefParamCtx, FuncDefParamKindCtx, FuncDefParamsCtx,
        FuncDefSelfRefParamCtx, FuncDefSelfValParamCtx,
    },
};
use tanitc_lexer::token::{lexeme::Lexeme, Token};
use tanitc_messages::Message;

use crate::{ParseResult, Parser};

impl Parser {
    pub fn parse_func_def_ctx(&mut self) -> ParseResult<FuncDefCtx> {
        let func_tkn = self.consume_token(Lexeme::KwFunc)?;
        let name_ctx = Box::new(self.parse_name_ctx()?);

        Ok(FuncDefCtx {
            func_tkn,
            params_ctx: self.parse_func_def_params_ctx(&name_ctx)?,
            name_ctx,
            return_type_ctx: self.parse_func_type_return_type_ctx()?,
            body_ctx: {
                let old_opt = self.does_ignore_nl();
                self.set_ignore_nl_option(false);

                let body_ctx = if !self.is_next(Lexeme::EndOfLine) {
                    Some(Box::new(self.parse_block_ctx()?))
                } else {
                    None
                };

                self.set_ignore_nl_option(old_opt);

                body_ctx
            },
            attributes_ctx: Box::default(),
        })
    }
}

impl Parser {
    fn parse_func_def_self_ref_param_ctx(&mut self) -> ParseResult<FuncDefSelfRefParamCtx> {
        Ok(FuncDefSelfRefParamCtx {
            ampersand_tkn: self.consume_token(Lexeme::Ampersand)?,
            mut_tkn: {
                if self.is_next(Lexeme::KwMut) {
                    Some(self.consume_token(Lexeme::KwMut)?)
                } else {
                    None
                }
            },
            self_tkn: self.consume_token(Lexeme::KwSelf)?,
        })
    }

    fn parse_func_def_self_val_param_ctx(
        &mut self,
        mut_tkn: Option<Token>,
    ) -> ParseResult<FuncDefSelfValParamCtx> {
        Ok(FuncDefSelfValParamCtx {
            mut_tkn,
            self_tkn: self.consume_token(Lexeme::KwSelf)?,
        })
    }

    fn parse_func_def_common_param_ctx(
        &mut self,
        mut_tkn: Option<Token>,
    ) -> ParseResult<FuncDefCommonParamCtx> {
        Ok(FuncDefCommonParamCtx {
            mut_tkn,
            name_ctx: Box::new(self.parse_name_ctx()?),
            colon_tkn: self.consume_token(Lexeme::Colon)?,
            type_ctx: Box::new(self.parse_type_ctx()?),
        })
    }

    fn parse_func_def_param_kind_ctx(&mut self) -> ParseResult<FuncDefParamKindCtx> {
        if self.is_next(Lexeme::Ampersand) {
            return self
                .parse_func_def_self_ref_param_ctx()
                .map(FuncDefParamKindCtx::SelfRef);
        }

        let mut_tkn = {
            if self.is_next(Lexeme::KwMut) {
                Some(self.consume_token(Lexeme::KwMut)?)
            } else {
                None
            }
        };

        let next = self.peek_token().ok_or(Message::reached_eof())?;
        match next.lexeme_ref() {
            Lexeme::KwSelf => self
                .parse_func_def_self_val_param_ctx(mut_tkn)
                .map(FuncDefParamKindCtx::SelfVal),
            Lexeme::Identifier(_) => self
                .parse_func_def_common_param_ctx(mut_tkn)
                .map(FuncDefParamKindCtx::CommonParam),
            _ => Err(Message::unexpected_token(&next, &[])),
        }
    }

    fn parse_func_def_params_ctx(&mut self, name_ctx: &NameCtx) -> ParseResult<FuncDefParamsCtx> {
        Ok(FuncDefParamsCtx {
            lparen_tkn: self.consume_token(Lexeme::LParen)?,
            params_ctx: {
                let mut params = Vec::<FuncDefParamCtx>::new();

                loop {
                    if self.is_next(Lexeme::RParen) {
                        break;
                    }

                    let param_ctx = self.parse_func_def_param_kind_ctx().map_err(|mut msg| {
                        msg.text = format!(
                            "In definition of function \"{}\": {}",
                            name_ctx.identifier(),
                            msg.text
                        );
                        msg
                    });

                    match param_ctx {
                        Ok(param_ctx) => {
                            let ctx = FuncDefParamCtx {
                                param_ctx,
                                comma_tkn: if self.is_next(Lexeme::Comma) {
                                    Some(self.consume_token(Lexeme::Comma)?)
                                } else {
                                    None
                                },
                            };

                            params.push(ctx);
                        }
                        Err(err) => {
                            self.error(err);
                            self.skip_until(&[Lexeme::Comma, Lexeme::RParen]);
                        }
                    }
                }

                params
            },
            rparen_tkn: self.consume_token(Lexeme::RParen)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use tanitc_ast::program_ctx::{
        statement_ctx::definition_ctx::func_def_ctx::FuncDefParamKindCtx,
        type_ctx::{tuple_type_ctx::TupleTypeCtx, TypeCtx},
    };
    use tanitc_lexer::token::lexeme::Lexeme;

    use crate::Parser;

    #[test]
    fn parse_func_def_test() {
        const SRC_TEXT: &str = "func hello(a: i32): () {\
                              \n    return\
                              \n}";

        let mut parser = Parser::from_text(SRC_TEXT);
        let func_def_ctx = parser.parse_func_def_ctx().unwrap();

        assert_eq!(func_def_ctx.name_ctx.to_string(), "hello");
        assert!(func_def_ctx.body_ctx.is_some());

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

        assert_eq!(func_def_ctx.params_ctx.params_ctx.len(), 1);
    }

    #[test]
    fn parse_func_def_with_self_test() {
        const SRC_TEXT: &str = "func with_self(self) { }";
        const FUNC_NAME: &str = "with_self";
        const PARAMS_COUNT: usize = 1;

        let mut parser = Parser::from_text(SRC_TEXT);
        let func_def_ctx = parser.parse_func_def_ctx().unwrap();

        assert_eq!(func_def_ctx.name_ctx.to_string(), FUNC_NAME);
        assert_eq!(func_def_ctx.params_ctx.params_ctx.len(), PARAMS_COUNT);

        let param_ctx = &func_def_ctx.params_ctx.params_ctx[0].param_ctx;
        let FuncDefParamKindCtx::SelfVal(self_val_param_ctx) = param_ctx else {
            panic!("Unexpected {}", param_ctx.kind_str())
        };
        assert!(self_val_param_ctx.mut_tkn.is_none());
    }

    #[test]
    fn parse_func_def_with_mut_self_test() {
        const SRC_TEXT: &str = "func with_mut_self(mut self) { }";
        const FUNC_NAME: &str = "with_mut_self";
        const PARAMS_COUNT: usize = 1;

        let mut parser = Parser::from_text(SRC_TEXT);
        let func_def_ctx = parser.parse_func_def_ctx().unwrap();

        assert_eq!(func_def_ctx.name_ctx.to_string(), FUNC_NAME);
        assert_eq!(func_def_ctx.params_ctx.params_ctx.len(), PARAMS_COUNT);

        let param_ctx = &func_def_ctx.params_ctx.params_ctx[0].param_ctx;
        let FuncDefParamKindCtx::SelfVal(self_val_param_ctx) = param_ctx else {
            panic!("Unexpected {}", param_ctx.kind_str())
        };
        assert!(self_val_param_ctx.mut_tkn.is_some());
    }

    #[test]
    fn parse_func_def_with_self_and_param_test() {
        const SRC_TEXT: &str = "func with_self_p(self, p: i32) { }";
        const FUNC_NAME: &str = "with_self_p";
        const PARAMS_COUNT: usize = 2;

        let mut parser = Parser::from_text(SRC_TEXT);
        let func_def_ctx = parser.parse_func_def_ctx().unwrap();

        assert_eq!(func_def_ctx.name_ctx.to_string(), FUNC_NAME);
        assert_eq!(func_def_ctx.params_ctx.params_ctx.len(), PARAMS_COUNT);

        {
            const PARAM_INDEX: usize = 0;

            let param_ctx = &func_def_ctx.params_ctx.params_ctx[PARAM_INDEX].param_ctx;
            let FuncDefParamKindCtx::SelfVal(self_val_param_ctx) = param_ctx else {
                panic!("Unexpected {}", param_ctx.kind_str())
            };

            assert!(self_val_param_ctx.mut_tkn.is_none());
        }

        {
            const PARAM_INDEX: usize = 1;
            const PARAM_NAME: &str = "p";
            const PARAM_TYPE_NAME: &str = "i32";

            let param_ctx = &func_def_ctx.params_ctx.params_ctx[PARAM_INDEX].param_ctx;
            let FuncDefParamKindCtx::CommonParam(param_ctx) = param_ctx else {
                panic!("Unexpected {}", param_ctx.kind_str())
            };

            assert!(param_ctx.mut_tkn.is_none());
            assert_eq!(param_ctx.name_ctx.to_string(), PARAM_NAME);
            assert_eq!(*param_ctx.colon_tkn.lexeme_ref(), Lexeme::Colon);

            let TypeCtx::Named(param_type_ctx) = param_ctx.type_ctx.as_ref() else {
                panic!("Unexpected {}", param_ctx.type_ctx.kind_str());
            };

            assert_eq!(param_type_ctx.name_ctx.to_string(), PARAM_TYPE_NAME);
            assert!(param_type_ctx.generic_ctx.is_none());
        }
    }

    #[test]
    fn parse_func_def_with_self_ref_test() {
        const SRC_TEXT: &str = "func with_self_ref(& self) { }";
        const FUNC_NAME: &str = "with_self_ref";
        const PARAMS_COUNT: usize = 1;

        let mut parser = Parser::from_text(SRC_TEXT);
        let func_def_ctx = parser.parse_func_def_ctx().unwrap();

        assert_eq!(func_def_ctx.name_ctx.to_string(), FUNC_NAME);
        assert_eq!(func_def_ctx.params_ctx.params_ctx.len(), PARAMS_COUNT);

        let param_ctx = &func_def_ctx.params_ctx.params_ctx[0].param_ctx;
        let FuncDefParamKindCtx::SelfRef(self_ref_param_ctx) = param_ctx else {
            panic!("Unexpected {}", param_ctx.kind_str())
        };
        assert!(self_ref_param_ctx.mut_tkn.is_none());
    }

    #[test]
    fn parse_func_def_with_mut_self_ref_test() {
        const SRC_TEXT: &str = "func with_mut_self_ref(&mut self) { }";
        const FUNC_NAME: &str = "with_mut_self_ref";
        const PARAMS_COUNT: usize = 1;

        let mut parser = Parser::from_text(SRC_TEXT);
        let func_def_ctx = parser.parse_func_def_ctx().unwrap();

        assert_eq!(func_def_ctx.name_ctx.to_string(), FUNC_NAME);
        assert_eq!(func_def_ctx.params_ctx.params_ctx.len(), PARAMS_COUNT);

        let param_ctx = &func_def_ctx.params_ctx.params_ctx[0].param_ctx;
        let FuncDefParamKindCtx::SelfRef(self_ref_param_ctx) = param_ctx else {
            panic!("Unexpected {}", param_ctx.kind_str())
        };
        assert!(self_ref_param_ctx.mut_tkn.is_some());
    }

    #[test]
    fn parse_bad_mut_param_test() {
        const SRC_TEXT: &str = "func with_mut_self(self mut) { }";

        const ERR_1: &str =
            "Syntax error: In definition of function \"with_mut_self\": Unexpected token: ')'. ";

        let mut parser = Parser::from_text(SRC_TEXT);

        let _ = parser.parse_func_def_ctx().unwrap();

        let errors = parser.messages_ref().errors_ref();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].text, ERR_1);
    }

    #[test]
    fn parse_bad_mut_self_ref_param_test() {
        const SRC_TEXT: &str = "func with_mut_self_ref(mut & self) { }";

        const ERR_1: &str =
            "Syntax error: In definition of function \"with_mut_self_ref\": Unexpected token: '&'. ";

        let mut parser = Parser::from_text(SRC_TEXT);

        let _ = parser.parse_func_def_ctx().unwrap();

        let errors = parser.messages_ref().errors_ref();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].text, ERR_1);
    }
}
