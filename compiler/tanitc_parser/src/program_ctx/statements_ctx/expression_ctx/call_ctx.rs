use tanitc_ast::program_ctx::statement_ctx::expression_ctx::{
    call_ctx::{CallCtx, CallParamCtx, CallParamsCtx, NamedCallParamCtx},
    ExpressionCtx,
};
use tanitc_lexer::token::{lexeme::Lexeme, Token};
use tanitc_messages::Message;

use crate::{ParseResult, Parser};

impl Parser {
    pub fn parse_call_ctx(&mut self, expression_ctx: Box<ExpressionCtx>) -> ParseResult<CallCtx> {
        Ok(CallCtx {
            expression_ctx,
            lparen_tkn: self.consume_token(Lexeme::LParen)?,
            params_ctx: self.parse_call_params_ctx()?,
            rparen_tkn: self.consume_token(Lexeme::RParen)?,
        })
    }

    pub fn parse_call_params_ctx(&mut self) -> ParseResult<CallParamsCtx> {
        let mut params = Vec::<(Option<CallParamCtx>, Option<Token>)>::new();

        loop {
            let Some(next) = self.peek_token() else {
                break;
            };

            let param_ctx = match next.lexeme_ref() {
                Lexeme::Rcb => break,
                Lexeme::EndOfLine => None,

                Lexeme::Identifier(_) => Some(CallParamCtx::Named(NamedCallParamCtx {
                    name_ctx: Box::new(self.parse_name_ctx()?),
                    colon_tkn: self.consume_token(Lexeme::Colon)?,
                    expression_ctx: Box::new(self.parse_expression_ctx()?),
                })),

                _ => {
                    self.error(Message::unexpected_token(&next, &[]));
                    self.skip_until(&[Lexeme::EndOfLine]);

                    continue;
                }
            };

            let nl_tkn = self.consume_token(Lexeme::EndOfLine).ok();

            params.push((param_ctx, nl_tkn));
        }

        Ok(CallParamsCtx { params })
    }
}
