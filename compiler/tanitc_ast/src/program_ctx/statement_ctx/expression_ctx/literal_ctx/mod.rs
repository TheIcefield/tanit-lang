use tanitc_lexer::token::Token;

pub mod array_literal_ctx;
pub mod struct_literal_ctx;
pub mod tuple_literal_ctx;

#[derive(Debug, Clone)]
pub enum LiteralCtx {
    Integer(Token),
    Decimal(Token),
    Text(Token),
    Array(array_literal_ctx::ArrayLiteralCtx),
    Tuple(tuple_literal_ctx::TupleLiteralCtx),
    Struct(struct_literal_ctx::StructLiteralCtx),
}
