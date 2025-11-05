use tanitc_ast::program_ctx::statement_ctx::expression_ctx::binary_ctx::{BinaryCtx, BinaryOpCtx};
use tanitc_hir::hir::expressions::{
    binary::{BinaryExpr, BinaryOperation},
    Expression,
};

use crate::{AstLowResult, AstLowering};

impl AstLowering {
    pub(crate) fn low_binary_expression_ctx(
        &mut self,
        ctx: &BinaryCtx,
    ) -> AstLowResult<BinaryExpr> {
        let operation = self.low_binary_operator(&ctx.binary_op_ctx);
        let lhs = Box::new(self.low_expression_ctx(&ctx.left_ctx)?);
        let rhs = Box::new(self.low_expression_ctx(&ctx.right_ctx)?);
        let location = lhs.location();

        // 'a += b' => 'a = a + b'
        if self.is_self_modify_operator(&ctx.binary_op_ctx) {
            let rhs = Box::new(Expression::Binary(BinaryExpr {
                location,
                operation,
                lhs: lhs.clone(),
                rhs,
            }));

            return Ok(BinaryExpr {
                location,
                operation: BinaryOperation::Assign,
                lhs,
                rhs,
            });
        }

        Ok(BinaryExpr {
            location,
            operation,
            lhs,
            rhs,
        })
    }

    fn low_binary_operator(&self, op: &BinaryOpCtx) -> BinaryOperation {
        match op {
            BinaryOpCtx::Add(_) => BinaryOperation::Add,
            BinaryOpCtx::Sub(_) => BinaryOperation::Sub,
            BinaryOpCtx::Mul(_) => BinaryOperation::Mul,
            BinaryOpCtx::Div(_) => BinaryOperation::Div,
            BinaryOpCtx::Mod(_) => BinaryOperation::Mod,
            BinaryOpCtx::BitAnd(_) => BinaryOperation::BitwiseAnd,
            BinaryOpCtx::LogicAnd(_) => BinaryOperation::LogicalAnd,
            BinaryOpCtx::BitOr(_) => BinaryOperation::BitwiseOr,
            BinaryOpCtx::LogicOr(_) => BinaryOperation::LogicalOr,
            BinaryOpCtx::BitXor(_) => BinaryOperation::BitwiseXor,
            BinaryOpCtx::Eq(_) => BinaryOperation::LogicalEq,
            BinaryOpCtx::Ne(_) => BinaryOperation::LogicalNe,
            BinaryOpCtx::Lt(_) => BinaryOperation::LogicalLt,
            BinaryOpCtx::Le(_) => BinaryOperation::LogicalLe,
            BinaryOpCtx::Gt(_) => BinaryOperation::LogicalGt,
            BinaryOpCtx::Ge(_) => BinaryOperation::LogicalGe,
            BinaryOpCtx::Shl(_) => BinaryOperation::ShiftL,
            BinaryOpCtx::Shr(_) => BinaryOperation::ShiftR,
            BinaryOpCtx::Assign(_) => BinaryOperation::Assign,
            BinaryOpCtx::AddAssign(_) => BinaryOperation::Add,
            BinaryOpCtx::SubAssign(_) => BinaryOperation::Sub,
            BinaryOpCtx::MulAssign(_) => BinaryOperation::Mul,
            BinaryOpCtx::DivAssign(_) => BinaryOperation::Div,
            BinaryOpCtx::ModAssign(_) => BinaryOperation::Mod,
            BinaryOpCtx::BitAndAssign(_) => BinaryOperation::BitwiseAnd,
            BinaryOpCtx::BitOrAssign(_) => BinaryOperation::BitwiseOr,
            BinaryOpCtx::BitXorAssign(_) => BinaryOperation::BitwiseXor,
            BinaryOpCtx::LeftShiftAssign(_) => BinaryOperation::ShiftL,
            BinaryOpCtx::RightShiftAssign(_) => BinaryOperation::ShiftR,
            BinaryOpCtx::Access(_) => BinaryOperation::Access,
            BinaryOpCtx::ScopeRes(_) => BinaryOperation::ScopeRes,
        }
    }

    fn is_self_modify_operator(&self, op: &BinaryOpCtx) -> bool {
        matches!(
            op,
            BinaryOpCtx::AddAssign(_)
                | BinaryOpCtx::SubAssign(_)
                | BinaryOpCtx::MulAssign(_)
                | BinaryOpCtx::DivAssign(_)
                | BinaryOpCtx::ModAssign(_)
                | BinaryOpCtx::BitAndAssign(_)
                | BinaryOpCtx::BitOrAssign(_)
                | BinaryOpCtx::BitXorAssign(_)
                | BinaryOpCtx::LeftShiftAssign(_)
                | BinaryOpCtx::RightShiftAssign(_)
        )
    }
}
