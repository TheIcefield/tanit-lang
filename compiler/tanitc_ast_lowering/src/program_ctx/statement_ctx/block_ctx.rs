use tanitc_ast::program_ctx::statement_ctx::{attributes_ctx::AttributesCtx, block_ctx::BlockCtx};
use tanitc_hir::hir::blocks::{Block, BlockAttributes};

use crate::{AstLowResult, AstLowering};

impl AstLowering {
    pub(crate) fn low_block_ctx(&mut self, block_ctx: &BlockCtx) -> AstLowResult<Block> {
        Ok(Block {
            location: block_ctx.lcb_tkn.get_location(),
            attributes: self.low_block_attributes(&block_ctx.attributes_ctx)?,
            statements: self.low_statements_ctx(&block_ctx.statements_ctx)?,
            is_global: false,
        })
    }

    fn low_block_attributes(&self, ctx: &AttributesCtx) -> AstLowResult<BlockAttributes> {
        self.expect_incompatible_attribute(&ctx.pub_tkn)?;

        Ok(BlockAttributes {
            safety: self.low_safety(&ctx.safe_tkn, &ctx.unsafe_tkn)?,
        })
    }
}
