use tanitc_lexer::token::Token;

pub mod array_literal_ctx;
pub mod struct_literal_ctx;
pub mod tuple_literal_ctx;

/// A literal value expression.
///
/// Scalar literals (`Integer`, `Decimal`, `Text`) store the original token.
/// Compound literals (`Array`, `Tuple`, `Struct`) contain their own sub-trees
/// with delimiters and element lists.
#[derive(Debug, Clone)]
pub enum LiteralCtx {
    /// An integer literal, e.g. `42`.
    Integer(Token),
    /// A decimal (floating-point) literal, e.g. `3.14`.
    Decimal(Token),
    /// A string literal, e.g. `"hello"`.
    Text(Token),
    /// An array literal, e.g. `[1, 2, 3]`.
    Array(array_literal_ctx::ArrayLiteralCtx),
    /// A tuple literal, e.g. `(1, "two")`.
    Tuple(tuple_literal_ctx::TupleLiteralCtx),
    /// A struct literal, e.g. `Point { x: 0.0, y: 1.0 }`.
    Struct(struct_literal_ctx::StructLiteralCtx),
}
