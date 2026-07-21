//! Concrete syntax tree (CST) for the Tanit programming language.
//!
//! This crate defines the node types that represent a parsed Tanit source file.
//! Every node preserves all tokens from the source (keywords, punctuation,
//! whitespace separators), making the tree fully lossless. The `Ctx` suffix
//! on every type stands for "context" — a convention used throughout the
//! Tanit compiler to distinguish parse-tree nodes from their higher-level
//! HIR counterparts.
//!
//! # Position in the compiler pipeline
//!
//! ```text
//! Source code  →  tanitc_lexer  →  tanitc_ast  →  tanitc_ast_lowering  →  tanitc_hir
//!                  (tokens)       (this crate)    (lower to HIR)         (typed IR)
//! ```
//!
//! The parser ([`tanitc_parser`](https://docs.rs/tanitc_parser)) consumes
//! tokens produced by [`tanitc_lexer`] and builds a [`program_ctx::ProgramCtx`]
//! — the root of the CST.
//!
//! # Crate structure
//!
//! The single public module [`program_ctx`] contains the entire tree:
//!
//! - [`program_ctx::ProgramCtx`] — root node, a list of statements.
//! - [`program_ctx::statement_ctx`] — statements, definitions, expressions, branches, and control flow.
//! - [`program_ctx::name_ctx`] — identifier and scoped-name representations.
//! - [`program_ctx::type_ctx`] — type annotations (named, reference, pointer, function, tuple, array, never).

pub mod program_ctx;
