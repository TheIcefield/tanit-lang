use tanitc_ast::program_ctx::ProgramCtx;

use crate::{ParseResult, Parser};

pub(crate) mod name_ctx;
pub(crate) mod statements_ctx;
pub(crate) mod type_ctx;

impl Parser {
    pub(crate) fn parse_program_ctx(&mut self) -> ParseResult<ProgramCtx> {
        Ok(ProgramCtx {
            statements_ctx: self.parse_statements_ctx()?,
        })
    }
}
