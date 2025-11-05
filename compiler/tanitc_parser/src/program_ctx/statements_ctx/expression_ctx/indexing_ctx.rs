use tanitc_ast::program_ctx::statement_ctx::expression_ctx::{
    indexing_ctx::{IndexCtx, IndexingCtx},
    ExpressionCtx,
};
use tanitc_lexer::token::lexeme::Lexeme;

use crate::{ParseResult, Parser};

impl Parser {
    pub fn parse_indexing_ctx(&mut self, lhs: Box<ExpressionCtx>) -> ParseResult<IndexingCtx> {
        Ok(IndexingCtx {
            expression_ctx: lhs,
            lsb_tkn: self.consume_token(Lexeme::Lsb)?,
            index_ctx: self.parse_index_ctx()?,
            rsb_tkn: self.consume_token(Lexeme::Rsb)?,
        })
    }

    fn parse_index_ctx(&mut self) -> ParseResult<IndexCtx> {
        Ok(IndexCtx {
            expression_ctx: Box::new(self.parse_expression_ctx()?),
        })
    }
}
