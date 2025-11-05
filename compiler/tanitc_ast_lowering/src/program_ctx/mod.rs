use tanitc_ast::program_ctx::ProgramCtx;
use tanitc_attributes::Safety;
use tanitc_hir::hir::{
    blocks::{Block, BlockAttributes},
    Hir,
};
use tanitc_lexer::location::Location;

use crate::{AstLowResult, AstLowering};

pub(crate) mod name_ctx;
pub(crate) mod statement_ctx;
pub(crate) mod type_ctx;

impl AstLowering {
    pub(crate) fn low_program_ctx(&mut self, program_ctx: &ProgramCtx) -> AstLowResult<Hir> {
        let statements = self.low_statements_ctx(&program_ctx.statements_ctx)?;

        let location = if let Some(first) = statements.first() {
            first.location()
        } else {
            Location::default()
        };

        Ok(Block {
            statements,
            location,
            is_global: true,
            attributes: BlockAttributes {
                safety: Safety::Safe,
            },
        }
        .into())
    }
}
