use tanitc_lexer::token::Token;

use crate::program_ctx::{
    name_ctx::NameCtx,
    statement_ctx::{attributes_ctx::AttributesCtx, block_ctx::BlockCtx},
    ProgramCtx,
};

#[derive(Debug, Clone)]
pub enum ModuleDefBodyCtx {
    Internal(Box<BlockCtx>),   // '{' statements* '}'
    External(Box<ProgramCtx>), // statements*
}

impl ModuleDefBodyCtx {
    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::Internal(_) => "module-def-body-internal-ctx",
            Self::External(_) => "module-def-body-external-ctx",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModuleDefCtx {
    pub attributes_ctx: Box<AttributesCtx>,
    pub def_tkn: Option<Token>, // ('def')?
    pub module_tkn: Token,      // 'module'
    pub name_ctx: Box<NameCtx>,
    pub body_ctx: ModuleDefBodyCtx,
}

impl ModuleDefCtx {
    pub fn is_external(&self) -> bool {
        matches!(self.body_ctx, ModuleDefBodyCtx::External(_))
    }
}
