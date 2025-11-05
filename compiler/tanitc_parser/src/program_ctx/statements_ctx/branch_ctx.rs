use tanitc_ast::program_ctx::statement_ctx::branch_ctx::{
    else_ctx::{ElseBodyCtx, ElseCtx},
    if_ctx::IfCtx,
    loop_ctx::LoopCtx,
    while_ctx::WhileCtx,
    BranchCtx,
};
use tanitc_lexer::token::lexeme::Lexeme;
use tanitc_messages::Message;

use crate::{ParseResult, Parser};

impl Parser {
    pub fn parse_branch_ctx(&mut self) -> ParseResult<BranchCtx> {
        let next = self.peek_token().ok_or(Message::reached_eof())?;
        match next.lexeme_ref() {
            Lexeme::KwLoop => self.parse_loop_ctx().map(BranchCtx::Loop),
            Lexeme::KwWhile => self.parse_while_ctx().map(BranchCtx::While),
            Lexeme::KwIf => self.parse_if_ctx().map(BranchCtx::If),
            Lexeme::KwElse => self.parse_else_ctx().map(BranchCtx::Else),
            _ => Err(Message::unexpected_token(&next, &[])),
        }
    }

    pub fn parse_loop_ctx(&mut self) -> ParseResult<LoopCtx> {
        Ok(LoopCtx {
            loop_tkn: self.consume_token(Lexeme::KwLoop)?,
            block_ctx: Box::new(self.parse_block_ctx()?),
        })
    }

    pub fn parse_while_ctx(&mut self) -> ParseResult<WhileCtx> {
        Ok(WhileCtx {
            while_tkn: self.consume_token(Lexeme::KwWhile)?,
            expression_ctx: Box::new(self.parse_expression_ctx()?),
            block_ctx: Box::new(self.parse_block_ctx()?),
        })
    }

    pub fn parse_if_ctx(&mut self) -> ParseResult<IfCtx> {
        Ok(IfCtx {
            if_tkn: self.consume_token(Lexeme::KwIf)?,
            expression_ctx: Box::new(self.parse_expression_ctx()?),
            block_ctx: Box::new(self.parse_block_ctx()?),
        })
    }

    pub fn parse_else_ctx(&mut self) -> ParseResult<ElseCtx> {
        let else_tkn = self.consume_token(Lexeme::KwElse)?;
        let body_ctx = self.parse_else_body_ctx()?;

        Ok(ElseCtx { else_tkn, body_ctx })
    }

    fn parse_else_body_ctx(&mut self) -> ParseResult<ElseBodyCtx> {
        let next = self.peek_token().ok_or(Message::reached_eof())?;

        match next.lexeme_ref() {
            Lexeme::KwIf => self
                .parse_if_ctx()
                .map(|ctx| ElseBodyCtx::If(Box::new(ctx))),
            Lexeme::Lcb => self
                .parse_block_ctx()
                .map(|ctx| ElseBodyCtx::Block(Box::new(ctx))),
            _ => Err(Message::unexpected_token(
                &next,
                &[Lexeme::KwIf, Lexeme::Lcb],
            )),
        }
    }
}
