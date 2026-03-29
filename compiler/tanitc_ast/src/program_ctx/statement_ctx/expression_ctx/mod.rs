use crate::program_ctx::name_ctx::NameSpecCtx;

pub mod binary_ctx;
pub mod call_ctx;
pub mod conversion_ctx;
pub mod indexing_ctx;
pub mod literal_ctx;
pub mod operand_ctx;
pub mod operator_ctx;
pub mod paren_ctx;
pub mod unary_ctx;

#[derive(Debug, Clone)]
pub enum ExpressionCtx {
    Binary(binary_ctx::BinaryCtx),
    Unary(unary_ctx::UnaryCtx),
    Conversion(conversion_ctx::ConversionCtx),
    Call(call_ctx::CallCtx),
    ParenCtx(paren_ctx::ParenCtx),
    Indexing(indexing_ctx::IndexingCtx),
    Literal(literal_ctx::LiteralCtx),
    Variable(NameSpecCtx),
}

impl ExpressionCtx {
    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::Binary(_) => "binary-expression-ctx",
            Self::Unary(_) => "unary-expression-ctx",
            Self::Conversion(_) => "conversion-ctx",
            Self::Call(_) => "call-ctx",
            Self::ParenCtx(_) => "paren-ctx",
            Self::Indexing(_) => "indexing-ctx",
            Self::Literal(_) => "literal-ctx",
            Self::Variable(_) => "variable-ctx",
        }
    }

    pub fn is_binary(&self) -> bool {
        matches!(self, ExpressionCtx::Binary(_))
    }

    pub fn is_unary(&self) -> bool {
        matches!(self, ExpressionCtx::Unary(_))
    }

    pub fn is_conversion(&self) -> bool {
        matches!(self, ExpressionCtx::Conversion(_))
    }

    pub fn is_call(&self) -> bool {
        matches!(self, ExpressionCtx::Call(_))
    }

    pub fn is_paren(&self) -> bool {
        matches!(self, ExpressionCtx::ParenCtx(_))
    }

    pub fn is_indexing(&self) -> bool {
        matches!(self, ExpressionCtx::Indexing(_))
    }

    pub fn is_literal(&self) -> bool {
        matches!(self, ExpressionCtx::Literal(_))
    }

    pub fn is_variable(&self) -> bool {
        matches!(self, ExpressionCtx::Variable(_))
    }
}
