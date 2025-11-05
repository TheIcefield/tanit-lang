use tanitc_ast::program_ctx::name_ctx::NameCtx;

use crate::{ParseResult, Parser};

impl Parser {
    pub fn parse_name_ctx(&mut self) -> ParseResult<NameCtx> {
        Ok(NameCtx {
            name_tkn: self.consume_identifier()?,
        })
    }
}
