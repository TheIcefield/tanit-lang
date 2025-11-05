use tanitc_lexer::token::Token;

use crate::program_ctx::statement_ctx::expression_ctx::ExpressionCtx;

#[derive(Debug, Clone)]
pub enum BinaryOpCtx {
    Add(Token),              // '+'
    Sub(Token),              // '-'
    Mul(Token),              // '*'
    Div(Token),              // '/'
    Mod(Token),              // '%'
    BitAnd(Token),           // '&'
    LogicAnd(Token),         // '&&'
    BitOr(Token),            // '|'
    LogicOr(Token),          // '||'
    BitXor(Token),           // '^'
    Eq(Token),               // '=='
    Ne(Token),               // '!='
    Lt(Token),               // '<'
    Le(Token),               // '<='
    Gt(Token),               // '>'
    Ge(Token),               // '>='
    Shl(Token),              // '<<'
    Shr(Token),              // '>>'
    Assign(Token),           // '='
    AddAssign(Token),        // '+='
    SubAssign(Token),        // '-='
    MulAssign(Token),        // '*='
    DivAssign(Token),        // '/='
    ModAssign(Token),        // '%='
    BitAndAssign(Token),     // '&='
    BitOrAssign(Token),      // '|='
    BitXorAssign(Token),     // '^='
    LeftShiftAssign(Token),  // '<<='
    RightShiftAssign(Token), // '>>='
    Access(Token),           // '.'
    ScopeRes(Token),         // '::'
}

#[derive(Debug, Clone)]
pub struct BinaryCtx {
    pub left_ctx: Box<ExpressionCtx>,
    pub binary_op_ctx: BinaryOpCtx,
    pub right_ctx: Box<ExpressionCtx>,
}
