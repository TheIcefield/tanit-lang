use tanitc_attributes::Safety;
use tanitc_lexer::location::Location;

use crate::hir::Hir;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct BlockAttributes {
    pub safety: Safety,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Block {
    pub location: Location,
    pub attributes: BlockAttributes,
    pub statements: Vec<Hir>,
    pub is_global: bool,
}

impl From<Block> for Hir {
    fn from(value: Block) -> Self {
        Self::Block(value)
    }
}
