use tanitc_ast::program_ctx::type_ctx::func_type_ctx::{
    FuncTypeCtx, FuncTypeParamCtx, FuncTypeParamsCtx, FuncTypeReturnTypeCtx,
};
use tanitc_lexer::token::lexeme::Lexeme;

use crate::{ParseResult, Parser};

impl Parser {
    pub(crate) fn parse_func_type_ctx(&mut self) -> ParseResult<FuncTypeCtx> {
        Ok(FuncTypeCtx {
            func_tkn: self.consume_token(Lexeme::KwFunc)?,
            params_ctx: self.parse_func_type_params_ctx()?,
            return_type: self.parse_func_type_return_type_ctx()?,
        })
    }

    pub(crate) fn parse_func_type_return_type_ctx(
        &mut self,
    ) -> ParseResult<Option<FuncTypeReturnTypeCtx>> {
        let old_opt = self.does_ignore_nl();
        self.set_ignore_nl_option(false);

        let return_type_ctx = if self.is_next(Lexeme::Colon) {
            Some(FuncTypeReturnTypeCtx {
                colon_tkn: self.consume_token(Lexeme::Colon)?,
                type_ctx: Box::new(self.parse_type_ctx()?),
            })
        } else {
            None
        };

        self.set_ignore_nl_option(old_opt);

        Ok(return_type_ctx)
    }

    fn parse_func_type_params_ctx(&mut self) -> ParseResult<FuncTypeParamsCtx> {
        Ok(FuncTypeParamsCtx {
            lparen_tkn: self.consume_token(Lexeme::LParen)?,
            parameters: self.parse_func_type_params()?,
            rparen_tkn: self.consume_token(Lexeme::RParen)?,
        })
    }

    fn parse_func_type_params(&mut self) -> ParseResult<Vec<FuncTypeParamCtx>> {
        let mut params = Vec::<FuncTypeParamCtx>::new();

        loop {
            if self.is_next(Lexeme::RParen) {
                break;
            }

            params.push(self.parse_func_type_param_ctx()?);
        }

        Ok(params)
    }

    fn parse_func_type_param_ctx(&mut self) -> ParseResult<FuncTypeParamCtx> {
        Ok(FuncTypeParamCtx {
            type_ctx: Box::new(self.parse_type_ctx()?),
            comma_tkn: self.consume_token(Lexeme::Comma).ok(),
        })
    }
}

#[cfg(test)]
mod tests {
    use tanitc_ast::program_ctx::type_ctx::{
        func_type_ctx::FuncTypeReturnTypeCtx, named_type_ctx::NamedTypeCtx, TypeCtx,
    };
    use tanitc_lexer::token::lexeme::Lexeme;

    use crate::Parser;

    #[test]
    fn parse_empty_func_type_test() {
        const SRC_TEXT: &str = "func()";

        let mut parser = Parser::from_text(SRC_TEXT);
        let func_type_ctx = parser.parse_func_type_ctx().unwrap();

        assert_eq!(
            *func_type_ctx.params_ctx.lparen_tkn.lexeme_ref(),
            Lexeme::LParen
        );
        assert_eq!(
            *func_type_ctx.params_ctx.rparen_tkn.lexeme_ref(),
            Lexeme::RParen
        );
        assert!(func_type_ctx.params_ctx.parameters.is_empty());
        assert!(func_type_ctx.return_type.is_none());
    }

    #[test]
    fn parse_empty_func_type_with_ret_test() {
        const SRC_TEXT: &str = "func():i32";

        let mut parser = Parser::from_text(SRC_TEXT);
        let func_type_ctx = parser.parse_func_type_ctx().unwrap();

        assert_eq!(*func_type_ctx.func_tkn.lexeme_ref(), Lexeme::KwFunc);

        assert_eq!(
            *func_type_ctx.params_ctx.lparen_tkn.lexeme_ref(),
            Lexeme::LParen
        );
        assert_eq!(
            *func_type_ctx.params_ctx.rparen_tkn.lexeme_ref(),
            Lexeme::RParen
        );
        assert!(func_type_ctx.params_ctx.parameters.is_empty());

        let Some(FuncTypeReturnTypeCtx {
            colon_tkn,
            type_ctx,
        }) = &func_type_ctx.return_type
        else {
            panic!("Unexpected return type");
        };

        assert_eq!(*colon_tkn.lexeme_ref(), Lexeme::Colon);

        let TypeCtx::Named(NamedTypeCtx {
            name_ctx,
            generic_ctx,
        }) = type_ctx.as_ref()
        else {
            panic!("Unexpected {}", type_ctx.kind_str())
        };

        assert_eq!(name_ctx.to_string(), "i32");
        assert!(generic_ctx.is_none());
    }

    #[test]
    fn parse_func_type_test() {
        const SRC_TEXT: &str = "func(i32)";

        let mut parser = Parser::from_text(SRC_TEXT);
        let func_type_ctx = parser.parse_func_type_ctx().unwrap();

        assert_eq!(*func_type_ctx.func_tkn.lexeme_ref(), Lexeme::KwFunc);
        assert_eq!(
            *func_type_ctx.params_ctx.lparen_tkn.lexeme_ref(),
            Lexeme::LParen
        );
        assert_eq!(
            *func_type_ctx.params_ctx.rparen_tkn.lexeme_ref(),
            Lexeme::RParen
        );
        assert_eq!(func_type_ctx.params_ctx.parameters.len(), 1);

        {
            let param_ctx = &func_type_ctx.params_ctx.parameters[0];
            let TypeCtx::Named(NamedTypeCtx {
                name_ctx,
                generic_ctx,
            }) = param_ctx.type_ctx.as_ref()
            else {
                panic!("Unexpected {}", param_ctx.type_ctx.kind_str())
            };
            assert_eq!(name_ctx.to_string(), "i32");
            assert!(generic_ctx.is_none());
        }

        assert!(func_type_ctx.return_type.is_none());
    }

    #[test]
    fn parse_func_type_with_ret_test() {
        const SRC_TEXT: &str = "func(i32):i32";

        let mut parser = Parser::from_text(SRC_TEXT);
        let func_type_ctx = parser.parse_func_type_ctx().unwrap();

        assert_eq!(*func_type_ctx.func_tkn.lexeme_ref(), Lexeme::KwFunc);
        assert_eq!(
            *func_type_ctx.params_ctx.lparen_tkn.lexeme_ref(),
            Lexeme::LParen
        );
        assert_eq!(
            *func_type_ctx.params_ctx.rparen_tkn.lexeme_ref(),
            Lexeme::RParen
        );
        assert_eq!(func_type_ctx.params_ctx.parameters.len(), 1);

        {
            let param_ctx = &func_type_ctx.params_ctx.parameters[0];
            let TypeCtx::Named(NamedTypeCtx {
                name_ctx,
                generic_ctx,
            }) = param_ctx.type_ctx.as_ref()
            else {
                panic!("Unexpected {}", param_ctx.type_ctx.kind_str())
            };
            assert_eq!(name_ctx.to_string(), "i32");
            assert!(generic_ctx.is_none());
        }

        let Some(FuncTypeReturnTypeCtx {
            colon_tkn,
            type_ctx,
        }) = &func_type_ctx.return_type
        else {
            panic!("Unexpected return type");
        };

        assert_eq!(*colon_tkn.lexeme_ref(), Lexeme::Colon);

        let TypeCtx::Named(NamedTypeCtx {
            name_ctx,
            generic_ctx,
        }) = type_ctx.as_ref()
        else {
            panic!("Unexpected {}", type_ctx.kind_str())
        };

        assert_eq!(name_ctx.to_string(), "i32");
        assert!(generic_ctx.is_none());
    }
}
