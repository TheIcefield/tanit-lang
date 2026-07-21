use tanitc_lexer::token::Token;

use crate::program_ctx::name_ctx::NameSpecCtx;

/// A `use` import statement.
///
/// References a qualified name that should be brought into scope.
///
/// # Example
///
/// ```tanit
/// use std::io::stdout
/// ```
#[derive(Debug, Clone)]
pub struct UseCtx {
    pub use_tkn: Token, // 'use'
    pub name_spec_ctx: NameSpecCtx,
}
