use crate::program_ctx::statement_ctx::StatementsCtx;

pub mod name_ctx;
pub mod statement_ctx;
pub mod type_ctx;

#[derive(Default, Debug, Clone)]
pub struct ProgramCtx {
    pub statements_ctx: StatementsCtx,
}
