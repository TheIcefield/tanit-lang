//! Top-level AST node and child modules.
//!
//! A parsed Tanit source file is represented as a [`ProgramCtx`] — a sequence
//! of statements. All other AST nodes (definitions, expressions, types, names)
//! live in the sub-modules re-exported here.

use crate::program_ctx::statement_ctx::StatementsCtx;

pub mod name_ctx;
pub mod statement_ctx;
pub mod type_ctx;

/// Root of the concrete syntax tree.
///
/// Every successfully parsed Tanit source file produces exactly one
/// `ProgramCtx`. It contains the ordered list of top-level statements
/// (definitions, expressions, use imports, etc.).
///
/// # Example
///
/// For the source:
/// ```tanit
/// struct Point { x: f32 }
/// func main() { }
/// ```
///
/// The `ProgramCtx` holds two statements: a struct definition and a
/// function definition.
#[derive(Default, Debug, Clone)]
pub struct ProgramCtx {
    pub statements_ctx: StatementsCtx,
}
