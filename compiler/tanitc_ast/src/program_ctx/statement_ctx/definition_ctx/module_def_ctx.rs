use tanitc_lexer::token::Token;

use crate::program_ctx::{
    name_ctx::NameCtx,
    statement_ctx::{attributes_ctx::AttributesCtx, block_ctx::BlockCtx},
    ProgramCtx,
};

/// The body of a module definition.
///
/// An `Internal` module has an explicit brace-delimited block. An `External`
/// module (file-level implicit module) contains a nested [`ProgramCtx`].
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

/// A module definition: `module Name { ... }` or `module Name`.
///
/// Modules group related definitions. An explicit module uses braces;
/// an external module (at file level) implicitly wraps the remaining
/// statements.
#[derive(Debug, Clone)]
pub struct ModuleDefCtx {
    pub attributes_ctx: Box<AttributesCtx>,
    pub def_tkn: Option<Token>, // ('def')?
    pub module_tkn: Token,      // 'module'
    pub name_ctx: Box<NameCtx>,
    pub body_ctx: ModuleDefBodyCtx,
}

impl ModuleDefCtx {
    /// Returns `true` if this is a file-level (external) module.
    pub fn is_external(&self) -> bool {
        matches!(self.body_ctx, ModuleDefBodyCtx::External(_))
    }
}
